use std::fmt;
use std::path::Path;

use near_primitives::views::AccountView;

use crate::error::ErrorKind;
use crate::rpc::query::{
    Query, ViewAccessKey, ViewAccessKeyList, ViewAccount, ViewCode, ViewFunction, ViewState,
};
use crate::types::{AccountId, InMemorySigner, NearToken, PublicKey, SecretKey};
use crate::{BlockHeight, CryptoHash, Network, Worker};

use crate::operations::{CallTransaction, CreateAccountTransaction, Transaction};
use crate::result::{Execution, ExecutionFinalResult, Result};

/// `Account` is directly associated to an account in the network provided by the
/// [`Worker`] that creates it. This type offers methods to interact with any
/// network, such as creating transactions and calling into contract functions.
#[derive(Clone)]
pub struct Account {
    signer: InMemorySigner,
    worker: Worker<dyn Network>,
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Account")
            .field("id", &self.signer.account_id)
            .finish()
    }
}

impl Account {
    /// Create a new account with the given path to the credentials JSON file
    pub fn from_file(
        path: impl AsRef<Path>,
        worker: &Worker<impl Network + 'static>,
    ) -> Result<Self> {
        let signer = InMemorySigner::from_file(path.as_ref())?;
        Ok(Self::new(signer, worker.clone().coerce()))
    }

    /// Create an [`Account`] object from an [`AccountId`] and [`SecretKey`].
    pub fn from_secret_key(
        id: AccountId,
        sk: SecretKey,
        worker: &Worker<impl Network + 'static>,
    ) -> Self {
        Self {
            signer: InMemorySigner::from_secret_key(id, sk),
            worker: worker.clone().coerce(),
        }
    }

    pub(crate) fn new(signer: InMemorySigner, worker: Worker<dyn Network>) -> Self {
        Self { signer, worker }
    }

    /// Grab the current account identifier
    pub fn id(&self) -> &AccountId {
        &self.signer.account_id
    }

    /// Grab the signer of the account. This signer is used to sign all transactions
    /// sent to the network.
    pub fn signer(&self) -> &InMemorySigner {
        &self.signer
    }

    /// Call a contract on the network specified within `worker`, and return
    /// a [`CallTransaction`] object that we will make use to populate the
    /// rest of the call details. Note that the current [`Account`]'s secret
    /// key is used as the signer of the transaction.
    pub fn call(&self, contract_id: &AccountId, function: &str) -> CallTransaction {
        CallTransaction::new(
            self.worker.clone(),
            contract_id.to_owned(),
            self.signer.clone(),
            function,
        )
    }

    /// View call to a specified contract function. Returns a result which can
    /// be deserialized into borsh or JSON.
    pub fn view(&self, contract_id: &AccountId, function: &str) -> Query<'_, ViewFunction> {
        self.worker.view(contract_id, function)
    }

    /// Transfer NEAR to an account specified by `receiver_id` with the amount
    /// specified by `amount`. Returns the execution details of this transaction
    pub async fn transfer_near(
        &self,
        receiver_id: &AccountId,
        amount: NearToken,
    ) -> Result<ExecutionFinalResult> {
        self.worker
            .transfer_near(self.signer(), receiver_id, amount)
            .await
    }

    /// Deletes the current account, and returns the execution details of this
    /// transaction. The beneficiary will receive the funds of the account deleted
    pub async fn delete_account(self, beneficiary_id: &AccountId) -> Result<ExecutionFinalResult> {
        self.worker
            .delete_account(self.id(), &self.signer, beneficiary_id)
            .await
    }

    /// Views the current account's details such as balance and storage usage.
    pub fn view_account(&self) -> Query<'_, ViewAccount> {
        self.worker.view_account(self.id())
    }

    /// Views the current accounts's access key, given the [`PublicKey`] associated to it.
    pub fn view_access_key(&self, pk: &PublicKey) -> Query<'_, ViewAccessKey> {
        Query::new(
            self.worker.client(),
            ViewAccessKey {
                account_id: self.id().clone(),
                public_key: pk.clone(),
            },
        )
    }

    /// Views all the [`AccessKey`]s of the current account. This will return a list of
    /// [`AccessKey`]s along with each associated [`PublicKey`].
    ///
    /// [`AccessKey`]: crate::types::AccessKey
    pub fn view_access_keys(&self) -> Query<'_, ViewAccessKeyList> {
        Query::new(
            self.worker.client(),
            ViewAccessKeyList {
                account_id: self.id().clone(),
            },
        )
    }

    /// Create a new sub account. Returns a [`CreateAccountTransaction`] object
    /// that we can make use of to fill out the rest of the details. The subaccount
    /// id will be in the form of: "{new_account_id}.{parent_account_id}"
    pub fn create_subaccount<'a, 'b>(
        &'a self,
        new_account_id: &'b str,
    ) -> CreateAccountTransaction<'a, 'b> {
        CreateAccountTransaction::new(
            &self.worker,
            self.signer.clone(),
            self.id().clone(),
            new_account_id,
        )
    }

    /// Deploy contract code or WASM bytes to the account, and return us a new
    /// [`Contract`] object that we can use to interact with the contract.
    pub async fn deploy(&self, wasm: &[u8]) -> Result<Execution<Contract>> {
        let outcome = self
            .worker
            .client()
            .deploy(&self.signer, self.id(), wasm.into())
            .await?;

        Ok(Execution {
            result: Contract::new(self.signer().clone(), self.worker.clone()),
            details: ExecutionFinalResult::from_view(outcome),
        })
    }

    /// Start a batch transaction, using the current account as the signer and
    /// making calls into the contract provided by `contract_id`. Returns a
    /// [`Transaction`] object that we can use to add Actions to the batched
    /// transaction. Call `transact` to send the batched transaction to the
    /// network.
    pub fn batch(&self, contract_id: &AccountId) -> Transaction {
        Transaction::new(
            self.worker.clone(),
            self.signer().clone(),
            contract_id.clone(),
        )
    }

    /// Store the credentials of this account locally in the directory provided.
    pub async fn store_credentials(&self, save_dir: impl AsRef<Path> + Send) -> Result<()> {
        let savepath = save_dir.as_ref();
        std::fs::create_dir_all(&save_dir).map_err(|e| ErrorKind::Io.custom(e))?;
        let savepath = savepath.join(format!("{}.json", self.id()));
        crate::rpc::tool::write_cred_to_file(&savepath, self.id(), &self.secret_key().0)
    }

    /// Get the keys of this account. The public key can be retrieved from the secret key.
    pub fn secret_key(&self) -> &SecretKey {
        &self.signer.secret_key
    }

    /// Sets the [`SecretKey`] of this account. Future transactions will be signed
    /// using this newly provided key.
    pub fn set_secret_key(&mut self, sk: SecretKey) {
        self.signer.secret_key = sk;
    }
}

/// `Contract` is directly associated to a contract in the network provided by the
/// [`Worker`] that creates it. This type offers methods to interact with any
/// network, such as creating transactions and calling into contract functions.
#[derive(Clone)]
pub struct Contract {
    pub(crate) account: Account,
}

impl fmt::Debug for Contract {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Contract")
            .field("id", self.account.id())
            .finish()
    }
}

impl Contract {
    /// Create a [`Contract`] object from an [`AccountId`] and [`SecretKey`].
    pub fn from_secret_key(
        id: AccountId,
        sk: SecretKey,
        worker: &Worker<impl Network + 'static>,
    ) -> Self {
        Self::account(Account::from_secret_key(id, sk, worker))
    }

    pub(crate) fn new(signer: InMemorySigner, worker: Worker<dyn Network>) -> Self {
        Self {
            account: Account::new(signer, worker),
        }
    }

    pub(crate) fn account(account: Account) -> Self {
        Self { account }
    }

    /// Grab the current contract's account identifier
    pub fn id(&self) -> &AccountId {
        self.account.id()
    }

    /// Treat this [`Contract`] object as an [`Account`] type. This does nothing
    /// on chain/network, and is merely allowing `Account::*` functions to be
    /// used from this `Contract`.
    pub fn as_account(&self) -> &Account {
        &self.account
    }

    /// Treat this [`Contract`] object as an [`Account`] type. This does nothing
    /// on chain/network, and is merely allowing `Account::*` functions to be
    /// used from this `Contract`.
    pub fn as_account_mut(&mut self) -> &mut Account {
        &mut self.account
    }

    /// Grab the signer of the account. This signer is used to sign all transactions
    /// sent to the network.
    pub fn signer(&self) -> &InMemorySigner {
        self.account.signer()
    }

    /// Call the current contract's function using the contract's own account
    /// secret key to do the signing. Returns a [`CallTransaction`] object that
    /// we will make use to populate the rest of the call details.
    ///
    /// If we want to make use of the contract's secret key as a signer to call
    /// into another contract, use `contract.as_account().call` instead.
    pub fn call(&self, function: &str) -> CallTransaction {
        self.account.call(self.id(), function)
    }

    /// Call a view function into the current contract. Returns a result which can
    /// be deserialized into borsh or JSON.
    pub fn view(&self, function: &str) -> Query<'_, ViewFunction> {
        self.account.view(self.id(), function)
    }

    /// View the WASM code bytes of this contract.
    pub fn view_code(&self) -> Query<'_, ViewCode> {
        self.account.worker.view_code(self.id())
    }

    /// View a contract's state map of key value pairs.
    pub fn view_state(&self) -> Query<'_, ViewState> {
        self.account.worker.view_state(self.id())
    }

    /// Views the current contract's details such as balance and storage usage.
    pub fn view_account(&self) -> Query<'_, ViewAccount> {
        self.account.worker.view_account(self.id())
    }

    /// Views the current contract's access key, given the [`PublicKey`] associated to it.
    pub fn view_access_key(&self, pk: &PublicKey) -> Query<'_, ViewAccessKey> {
        self.account.view_access_key(pk)
    }

    /// Views all the [`AccessKey`]s of the current contract. This will return a list of
    /// [`AccessKey`]s along with each associated [`PublicKey`].
    ///
    /// [`AccessKey`]: crate::types::AccessKey
    pub fn view_access_keys(&self) -> Query<'_, ViewAccessKeyList> {
        self.account.view_access_keys()
    }

    /// Deletes the current contract, and returns the execution details of this
    /// transaction. The beneficiary will receive the funds of the account deleted
    pub async fn delete_contract(self, beneficiary_id: &AccountId) -> Result<ExecutionFinalResult> {
        self.account.delete_account(beneficiary_id).await
    }

    /// Start a batch transaction, using the current contract's secret key as the
    /// signer, making calls into itself. Returns a [`Transaction`] object that
    /// we can use to add Actions to the batched transaction. Call `transact`
    /// to send the batched transaction to the network.
    pub fn batch(&self) -> Transaction {
        self.account.batch(self.id())
    }
}

/// Details of an Account or Contract. This is an non-exhaustive list of items
/// that the account stores in the blockchain state.
///
/// This struct is the same as [`AccountDetails`] with the exception that it provides
/// optional fields that guard against 'null' overwrites when making a patch.
#[derive(Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct AccountDetailsPatch {
    pub balance: Option<NearToken>,
    pub locked: Option<NearToken>,
    pub code_hash: Option<CryptoHash>,
    pub storage_usage: Option<u64>,
    pub(crate) storage_paid_at: Option<BlockHeight>,
}

impl AccountDetailsPatch {
    pub fn reduce(&mut self, acc: Self) {
        if let Some(balance) = acc.balance {
            self.balance = Some(balance);
        }
        if let Some(locked) = acc.locked {
            self.locked = Some(locked);
        }
        if let Some(code_hash) = acc.code_hash {
            self.code_hash = Some(code_hash);
        }
        if let Some(storage) = acc.storage_usage {
            self.storage_usage = Some(storage);
        }
        if let Some(storage_paid_at) = acc.storage_paid_at {
            self.storage_paid_at = Some(storage_paid_at);
        }
    }

    pub fn balance(mut self, balance: NearToken) -> Self {
        self.balance = Some(balance);
        self
    }

    pub fn locked(mut self, locked: NearToken) -> Self {
        self.locked = Some(locked);
        self
    }

    pub fn code_hash(mut self, code_hash: CryptoHash) -> Self {
        self.code_hash = Some(code_hash);
        self
    }

    pub fn storage_usage(mut self, storage_usage: u64) -> Self {
        self.storage_usage = Some(storage_usage);
        self
    }
}

impl From<AccountDetails> for AccountDetailsPatch {
    fn from(account: AccountDetails) -> Self {
        Self {
            balance: Some(account.balance),
            locked: Some(account.locked),
            code_hash: Some(account.code_hash),
            storage_usage: Some(account.storage_usage),
            storage_paid_at: Some(account.storage_paid_at),
        }
    }
}

/// Details of an Account or Contract. This is an non-exhaustive list of items
/// that the account stores in the blockchain state.
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct AccountDetails {
    pub balance: NearToken,
    pub locked: NearToken,
    pub code_hash: CryptoHash,
    pub storage_usage: u64,
    // Deprecated value. Mainly used to be able to convert back into an AccountView
    pub(crate) storage_paid_at: BlockHeight,
}

impl AccountDetails {
    pub fn new() -> Self {
        Self {
            balance: NearToken::from_near(0),
            locked: NearToken::from_near(0),
            code_hash: CryptoHash::default(),
            storage_usage: 0,
            storage_paid_at: 0,
        }
    }

    pub(crate) fn into_near_account(self) -> near_primitives::account::Account {
        near_primitives::account::Account::new(
            self.balance.as_yoctonear(),
            self.locked.as_yoctonear(),
            near_primitives::hash::CryptoHash(self.code_hash.0),
            self.storage_usage,
        )
    }
}

impl Default for AccountDetails {
    fn default() -> Self {
        Self::new()
    }
}

impl From<AccountView> for AccountDetails {
    fn from(account: AccountView) -> Self {
        Self {
            balance: NearToken::from_yoctonear(account.amount),
            locked: NearToken::from_yoctonear(account.locked),
            code_hash: CryptoHash(account.code_hash.0),
            storage_usage: account.storage_usage,
            storage_paid_at: account.storage_paid_at,
        }
    }
}

impl From<AccountDetailsPatch> for AccountDetails {
    fn from(value: AccountDetailsPatch) -> Self {
        Self {
            balance: value.balance.unwrap_or_default(),
            locked: value.locked.unwrap_or_default(),
            code_hash: value.code_hash.unwrap_or_default(),
            storage_usage: value.storage_usage.unwrap_or_default(),
            storage_paid_at: value.storage_paid_at.unwrap_or_default(),
        }
    }
}
