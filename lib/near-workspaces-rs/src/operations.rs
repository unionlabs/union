//! All operation types that are generated/used when making transactions or view calls.

use crate::error::{ErrorKind, RpcErrorCode};
use crate::result::{Execution, ExecutionFinalResult, Result, ViewResultDetails};
use crate::rpc::client::{
    send_batch_tx_and_retry, send_batch_tx_async_and_retry, DEFAULT_CALL_DEPOSIT,
    DEFAULT_CALL_FN_GAS,
};
use crate::rpc::query::{Query, ViewFunction};
use crate::types::{
    AccessKey, AccountId, Gas, InMemorySigner, KeyType, NearToken, PublicKey, SecretKey,
};
use crate::worker::Worker;
use crate::{Account, CryptoHash, Network};

use near_account_id::ParseAccountError;
use near_gas::NearGas;
use near_jsonrpc_client::errors::{JsonRpcError, JsonRpcServerError};
use near_jsonrpc_client::methods::tx::RpcTransactionError;
use near_primitives::borsh;
use near_primitives::transaction::{
    Action, AddKeyAction, CreateAccountAction, DeleteAccountAction, DeleteKeyAction,
    DeployContractAction, FunctionCallAction, StakeAction, TransferAction,
};
use near_primitives::views::FinalExecutionOutcomeView;
use std::convert::TryInto;
use std::fmt;
use std::future::IntoFuture;
use std::pin::Pin;
use std::task::Poll;

const MAX_GAS: NearGas = NearGas::from_tgas(300);

/// A set of arguments we can provide to a transaction, containing
/// the function name, arguments, the amount of gas to use and deposit.
#[derive(Debug)]
pub struct Function {
    pub(crate) name: String,
    pub(crate) args: Result<Vec<u8>>,
    pub(crate) deposit: NearToken,
    pub(crate) gas: Gas,
}

impl Function {
    /// Initialize a new instance of [`Function`], tied to a specific function on a
    /// contract that lives directly on a contract we've specified in [`Transaction`].
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            args: Ok(vec![]),
            deposit: DEFAULT_CALL_DEPOSIT,
            gas: DEFAULT_CALL_FN_GAS,
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    pub fn args(mut self, args: Vec<u8>) -> Self {
        if self.args.is_err() {
            return self;
        }
        self.args = Ok(args);
        self
    }

    /// Similar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Self {
        match serde_json::to_vec(&args) {
            Ok(args) => self.args = Ok(args),
            Err(e) => self.args = Err(ErrorKind::DataConversion.custom(e)),
        }
        self
    }

    /// Similar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    pub fn args_borsh<U: borsh::BorshSerialize>(mut self, args: U) -> Self {
        match borsh::to_vec(&args) {
            Ok(args) => self.args = Ok(args),
            Err(e) => self.args = Err(ErrorKind::DataConversion.custom(e)),
        }
        self
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    pub fn deposit(mut self, deposit: NearToken) -> Self {
        self.deposit = deposit;
        self
    }

    /// Specify the amount of gas to be used.
    pub fn gas(mut self, gas: Gas) -> Self {
        self.gas = gas;
        self
    }

    /// Use the maximum amount of gas possible to perform this function call into the contract.
    pub fn max_gas(self) -> Self {
        self.gas(MAX_GAS)
    }
}

/// A builder-like object that will allow specifying various actions to be performed
/// in a single transaction. For details on each of the actions, find them in
/// [NEAR transactions](https://docs.near.org/docs/concepts/transaction).
///
/// All actions are performed on the account specified by `receiver_id`. This object
/// is most commonly constructed from [`Account::batch`] or [`Contract::batch`],
/// where `receiver_id` is specified in the `Account::batch` while `Contract::id()`
/// is used by default for `Contract::batch`.
///
/// [`Contract::batch`]: crate::Contract::batch
pub struct Transaction {
    worker: Worker<dyn Network>,
    signer: InMemorySigner,
    receiver_id: AccountId,
    // Result used to defer errors in argument parsing to later when calling into transact
    actions: Result<Vec<Action>>,
}

impl Transaction {
    pub(crate) fn new(
        worker: Worker<dyn Network>,
        signer: InMemorySigner,
        receiver_id: AccountId,
    ) -> Self {
        Self {
            worker,
            signer,
            receiver_id,
            actions: Ok(Vec::new()),
        }
    }

    /// Adds a key to the `receiver_id`'s account, where the public key can be used
    /// later to delete the same key.
    pub fn add_key(mut self, pk: PublicKey, ak: AccessKey) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(
                AddKeyAction {
                    public_key: pk.into(),
                    access_key: ak.into(),
                }
                .into(),
            );
        }

        self
    }

    /// Call into the `receiver_id`'s contract with the specific function arguments.
    pub fn call(mut self, function: Function) -> Self {
        let args = match function.args {
            Ok(args) => args,
            Err(err) => {
                self.actions = Err(err);
                return self;
            }
        };

        if let Ok(actions) = &mut self.actions {
            actions.push(Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: function.name.to_string(),
                args,
                deposit: function.deposit.as_yoctonear(),
                gas: function.gas.as_gas(),
            })));
        }

        self
    }

    /// Create a new account with the account id being `receiver_id`.
    pub fn create_account(mut self) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(CreateAccountAction {}.into());
        }
        self
    }

    /// Deletes the `receiver_id`'s account. The beneficiary specified by
    /// `beneficiary_id` will receive the funds of the account deleted.
    pub fn delete_account(mut self, beneficiary_id: &AccountId) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(
                DeleteAccountAction {
                    beneficiary_id: beneficiary_id.clone(),
                }
                .into(),
            );
        }
        self
    }

    /// Deletes a key from the `receiver_id`'s account, where the public key is
    /// associated with the access key to be deleted.
    pub fn delete_key(mut self, pk: PublicKey) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(DeleteKeyAction { public_key: pk.0 }.into());
        }
        self
    }

    /// Deploy contract code or WASM bytes to the `receiver_id`'s account.
    pub fn deploy(mut self, code: &[u8]) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(DeployContractAction { code: code.into() }.into());
        }
        self
    }

    /// An action which stakes the signer's tokens and setups a validator public key.
    pub fn stake(mut self, stake: NearToken, pk: PublicKey) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(
                StakeAction {
                    stake: stake.as_yoctonear(),
                    public_key: pk.0,
                }
                .into(),
            );
        }
        self
    }

    /// Transfer `deposit` amount from `signer`'s account into `receiver_id`'s account.
    pub fn transfer(mut self, deposit: NearToken) -> Self {
        if let Ok(actions) = &mut self.actions {
            actions.push(
                TransferAction {
                    deposit: deposit.as_yoctonear(),
                }
                .into(),
            );
        }
        self
    }

    async fn transact_raw(self) -> Result<FinalExecutionOutcomeView> {
        let view = send_batch_tx_and_retry(
            self.worker.client(),
            &self.signer,
            &self.receiver_id,
            self.actions?,
        )
        .await?;

        if !self.worker.tx_callbacks.is_empty() {
            let total_gas_burnt = view.transaction_outcome.outcome.gas_burnt
                + view
                    .receipts_outcome
                    .iter()
                    .map(|t| t.outcome.gas_burnt)
                    .sum::<u64>();

            for callback in self.worker.tx_callbacks {
                callback(Gas::from_gas(total_gas_burnt))?;
            }
        }

        Ok(view)
    }

    /// Process the transaction, and return the result of the execution.
    pub async fn transact(self) -> Result<ExecutionFinalResult> {
        self.transact_raw()
            .await
            .map(ExecutionFinalResult::from_view)
            .map_err(crate::error::Error::from)
    }

    /// Send the transaction to the network to be processed. This will be done asynchronously
    /// without waiting for the transaction to complete. This returns us a [`TransactionStatus`]
    /// for which we can call into [`status`] and/or `.await` to retrieve info about whether
    /// the transaction has been completed or not. Note that `.await` will wait till completion
    /// of the transaction.
    ///
    /// [`status`]: TransactionStatus::status
    pub async fn transact_async(self) -> Result<TransactionStatus> {
        send_batch_tx_async_and_retry(self.worker, &self.signer, &self.receiver_id, self.actions?)
            .await
    }
}

/// Similar to a [`Transaction`], but more specific to making a call into a contract.
/// Note, only one call can be made per `CallTransaction`.
pub struct CallTransaction {
    worker: Worker<dyn Network>,
    signer: InMemorySigner,
    contract_id: AccountId,
    function: Function,
}

impl CallTransaction {
    pub(crate) fn new(
        worker: Worker<dyn Network>,
        contract_id: AccountId,
        signer: InMemorySigner,
        function: &str,
    ) -> Self {
        Self {
            worker,
            signer,
            contract_id,
            function: Function::new(function),
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    pub fn args(mut self, args: Vec<u8>) -> Self {
        self.function = self.function.args(args);
        self
    }

    /// Similar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Self {
        self.function = self.function.args_json(args);
        self
    }

    /// Similar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    pub fn args_borsh<U: borsh::BorshSerialize>(mut self, args: U) -> Self {
        self.function = self.function.args_borsh(args);
        self
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    pub fn deposit(mut self, deposit: NearToken) -> Self {
        self.function = self.function.deposit(deposit);
        self
    }

    /// Specify the amount of gas to be used where `gas` is the amount of gas in yocto near.
    pub fn gas(mut self, gas: NearGas) -> Self {
        self.function = self.function.gas(gas);
        self
    }

    /// Use the maximum amount of gas possible to perform this transaction.
    pub fn max_gas(self) -> Self {
        self.gas(MAX_GAS)
    }

    /// Finally, send the transaction to the network. This will consume the `CallTransaction`
    /// object and return us the execution details, along with any errors if the transaction
    /// failed in any process along the way.
    pub async fn transact(self) -> Result<ExecutionFinalResult> {
        let txn = self
            .worker
            .client()
            .call(
                &self.signer,
                &self.contract_id,
                self.function.name.to_string(),
                self.function.args?,
                self.function.gas.as_gas(),
                self.function.deposit,
            )
            .await
            .map(ExecutionFinalResult::from_view)
            .map_err(crate::error::Error::from)?;

        for callback in self.worker.tx_callbacks.iter() {
            callback(txn.total_gas_burnt)?;
        }
        Ok(txn)
    }

    /// Send the transaction to the network to be processed. This will be done asynchronously
    /// without waiting for the transaction to complete. This returns us a [`TransactionStatus`]
    /// for which we can call into [`status`] and/or `.await` to retrieve info about whether
    /// the transaction has been completed or not. Note that `.await` will wait till completion
    /// of the transaction.
    ///
    /// [`status`]: TransactionStatus::status
    pub async fn transact_async(self) -> Result<TransactionStatus> {
        send_batch_tx_async_and_retry(
            self.worker,
            &self.signer,
            &self.contract_id,
            vec![FunctionCallAction {
                args: self.function.args?,
                method_name: self.function.name,
                gas: self.function.gas.as_gas(),
                deposit: self.function.deposit.as_yoctonear(),
            }
            .into()],
        )
        .await
    }

    /// Instead of transacting the transaction, call into the specified view function.
    pub async fn view(self) -> Result<ViewResultDetails> {
        Query::new(
            self.worker.client(),
            ViewFunction {
                account_id: self.contract_id.clone(),
                function: self.function,
            },
        )
        .await
    }
}

/// Similar to a [`Transaction`], but more specific to creating an account.
/// This transaction will create a new account with the specified `receiver_id`
pub struct CreateAccountTransaction<'a, 'b> {
    worker: &'a Worker<dyn Network>,
    signer: InMemorySigner,
    parent_id: AccountId,
    new_account_id: &'b str,

    initial_balance: NearToken,
    secret_key: Option<SecretKey>,
}

impl<'a, 'b> CreateAccountTransaction<'a, 'b> {
    pub(crate) fn new(
        worker: &'a Worker<dyn Network>,
        signer: InMemorySigner,
        parent_id: AccountId,
        new_account_id: &'b str,
    ) -> Self {
        Self {
            worker,
            signer,
            parent_id,
            new_account_id,
            initial_balance: NearToken::from_yoctonear(100000000000000000000000u128),
            secret_key: None,
        }
    }

    /// Specifies the initial balance of the new account. Amount directly taken out
    /// from the caller/signer of this transaction.
    pub fn initial_balance(mut self, initial_balance: NearToken) -> Self {
        self.initial_balance = initial_balance;
        self
    }

    /// Set the secret key of the new account.
    pub fn keys(mut self, secret_key: SecretKey) -> Self {
        self.secret_key = Some(secret_key);
        self
    }

    /// Send the transaction to the network. This will consume the `CreateAccountTransaction`
    /// and give us back the details of the execution and finally the new [`Account`] object.
    pub async fn transact(self) -> Result<Execution<Account>> {
        let sk = self
            .secret_key
            .unwrap_or_else(|| SecretKey::from_seed(KeyType::ED25519, "subaccount.seed"));
        let id: AccountId = format!("{}.{}", self.new_account_id, self.parent_id)
            .try_into()
            .map_err(|e: ParseAccountError| ErrorKind::DataConversion.custom(e))?;

        let outcome = self
            .worker
            .client()
            .create_account(&self.signer, &id, sk.public_key(), self.initial_balance)
            .await?;

        let signer = InMemorySigner::from_secret_key(id, sk);
        let account = Account::new(signer, self.worker.clone());
        let details = ExecutionFinalResult::from_view(outcome);

        for callback in self.worker.tx_callbacks.iter() {
            callback(details.total_gas_burnt)?;
        }

        Ok(Execution {
            result: account,
            details,
        })
    }
}

/// `TransactionStatus` object relating to an [`asynchronous transaction`] on the network.
/// Used to query into the status of the Transaction for whether it has completed or not.
///
/// [`asynchronous transaction`]: https://docs.near.org/api/rpc/transactions#send-transaction-async
#[must_use]
pub struct TransactionStatus {
    worker: Worker<dyn Network>,
    sender_id: AccountId,
    hash: CryptoHash,
}

impl TransactionStatus {
    pub(crate) fn new(
        worker: Worker<dyn Network>,
        id: AccountId,
        hash: near_primitives::hash::CryptoHash,
    ) -> Self {
        Self {
            worker,
            sender_id: id,
            hash: CryptoHash(hash.0),
        }
    }

    /// Checks the status of the transaction. If an `Err` is returned, then the transaction
    /// is in an unexpected state. The error should have further context. Otherwise, if an
    /// `Ok` value with [`Poll::Pending`] is returned, then the transaction has not finished.
    pub async fn status(&self) -> Result<Poll<ExecutionFinalResult>> {
        let result = self
            .worker
            .client()
            .tx_async_status(
                &self.sender_id,
                near_primitives::hash::CryptoHash(self.hash.0),
            )
            .await
            .map(ExecutionFinalResult::from_view);

        match result {
            Ok(result) => Ok(Poll::Ready(result)),
            Err(err) => match err {
                JsonRpcError::ServerError(JsonRpcServerError::HandlerError(
                    RpcTransactionError::UnknownTransaction { .. },
                )) => Ok(Poll::Pending),
                other => Err(RpcErrorCode::BroadcastTxFailure.custom(other)),
            },
        }
    }

    /// Wait until the completion of the transaction by polling [`TransactionStatus::status`].
    pub(crate) async fn wait(self) -> Result<ExecutionFinalResult> {
        loop {
            match self.status().await? {
                Poll::Ready(val) => break Ok(val),
                Poll::Pending => (),
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }
    }

    /// Get the [`AccountId`] of the account that initiated this transaction.
    pub fn sender_id(&self) -> &AccountId {
        &self.sender_id
    }

    /// Reference [`CryptoHash`] to the submitted transaction, pending completion.
    pub fn hash(&self) -> &CryptoHash {
        &self.hash
    }
}

impl fmt::Debug for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransactionStatus")
            .field("sender_id", &self.sender_id)
            .field("hash", &self.hash)
            .finish()
    }
}

impl IntoFuture for TransactionStatus {
    type Output = Result<ExecutionFinalResult>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output>>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async { self.wait().await })
    }
}
