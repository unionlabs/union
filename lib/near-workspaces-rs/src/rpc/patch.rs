use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::state_record::StateRecord;
use near_primitives::types::{BlockId, BlockReference};
use near_token::NearToken;

use crate::error::SandboxErrorCode;
use crate::network::{Sandbox, DEV_ACCOUNT_SEED};
use crate::types::account::AccountDetails;
use crate::types::{BlockHeight, KeyType, PublicKey, SecretKey};
use crate::{AccessKey, AccountDetailsPatch, Result};
use crate::{AccountId, Contract, CryptoHash, InMemorySigner, Network, Worker};

/// A [`Transaction`]-like object that allows us to specify details about importing
/// a contract from a different network into our sandbox local network. This creates
/// a new [`Transaction`] to be committed to the sandbox network once `transact()`
/// has been called. This does not commit any new transactions from the network
/// this object is importing from.
///
/// [`Transaction`]: crate::operations::Transaction
pub struct ImportContractTransaction<'a> {
    account_id: &'a AccountId,
    from_network: Worker<dyn Network>,
    into_network: Worker<Sandbox>,

    /// Whether to grab data down from the other contract or not
    import_data: bool,

    /// Initial balance of the account. If None, uses what is specified
    /// from the other account instead.
    initial_balance: Option<NearToken>,

    block_ref: Option<BlockReference>,

    /// AccountId if specified, will be the destination account to clone the contract to.
    into_account_id: Option<AccountId>,
}

impl<'a> ImportContractTransaction<'a> {
    pub(crate) fn new(
        account_id: &'a AccountId,
        from_network: Worker<dyn Network>,
        into_network: Worker<Sandbox>,
    ) -> Self {
        ImportContractTransaction {
            account_id,
            from_network,
            into_network,
            import_data: false,
            initial_balance: None,
            block_ref: None,
            into_account_id: None,
        }
    }

    /// Specify at which block height to import the contract from. This is usable with
    /// any network this object is importing from, but be aware that only archival
    /// networks will have the full history while networks like mainnet or testnet
    /// only has the history from 5 or less epochs ago.
    pub fn block_height(mut self, block_height: BlockHeight) -> Self {
        self.block_ref = Some(BlockId::Height(block_height).into());
        self
    }

    /// Specify at which block hash to import the contract from. This is usable with
    /// any network this object is importing from, but be aware that only archival
    /// networks will have the full history while networks like mainnet or testnet
    /// only has the history from 5 or less epochs ago.
    pub fn block_hash(mut self, block_hash: CryptoHash) -> Self {
        self.block_ref =
            Some(BlockId::Hash(near_primitives::hash::CryptoHash(block_hash.0)).into());
        self
    }

    /// Along with importing the contract code, this will import the state from the
    /// contract itself. This is useful for testing current network state or state
    /// at a specific block. Note that there is a limit of 50kb of state data that
    /// can be pulled down using the usual RPC service. To get beyond this, our own
    /// RPC node has to be spun up and used instead.
    pub fn with_data(mut self) -> Self {
        self.import_data = true;
        self
    }

    /// Specifies the balance of the contract. This will override the balance currently
    /// on the network this transaction is importing from.
    pub fn initial_balance(mut self, initial_balance: NearToken) -> Self {
        self.initial_balance = Some(initial_balance);
        self
    }

    /// Sets the destination [`AccountId`] where the import will be transacted to.
    /// This function is provided so users can import to a different [`AccountId`]
    /// than the one initially provided to import from.
    pub fn dest_account_id(mut self, account_id: &AccountId) -> Self {
        self.into_account_id = Some(account_id.clone());
        self
    }

    /// Process the transaction, and return the result of the execution.
    pub async fn transact(self) -> Result<Contract> {
        let from_account_id = self.account_id;
        let into_account_id = self.into_account_id.as_ref().unwrap_or(from_account_id);

        let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
        let pk = sk.public_key();
        let signer = InMemorySigner::from_secret_key(into_account_id.clone(), sk);
        let block_ref = self.block_ref.unwrap_or_else(BlockReference::latest);

        let mut account_view = self
            .from_network
            .view_account(from_account_id)
            .block_reference(block_ref.clone())
            .await?;

        let code_hash = account_view.code_hash;
        if let Some(initial_balance) = self.initial_balance {
            account_view.balance = initial_balance;
        }

        let mut patch = PatchTransaction::new(&self.into_network, into_account_id.clone())
            .account(account_view.into())
            .access_key(pk, AccessKey::full_access());

        if code_hash != CryptoHash::default() {
            let code = self
                .from_network
                .view_code(from_account_id)
                .block_reference(block_ref.clone())
                .await?;
            patch = patch.code(&code);
        }

        if self.import_data {
            let states = self
                .from_network
                .view_state(from_account_id)
                .block_reference(block_ref)
                .await?.data;

            patch = patch.states(
                states
                    .iter()
                    .map(|(key, value)| (key.as_slice(), value.as_slice())),
            );
        }

        patch.transact().await?;
        Ok(Contract::new(signer, self.into_network.coerce()))
    }
}

/// Internal enum for determining whether to update the account on chain
/// or to patch an entire account.
enum AccountUpdate {
    Update(AccountDetailsPatch),
    FromCurrent(Box<dyn Fn(AccountDetails) -> AccountDetailsPatch + Send>),
}

pub struct PatchTransaction {
    account_id: AccountId,
    records: Vec<StateRecord>,
    worker: Worker<Sandbox>,
    account_updates: Vec<AccountUpdate>,
    code_hash_update: Option<CryptoHash>,
}

impl PatchTransaction {
    pub(crate) fn new(worker: &Worker<Sandbox>, account_id: AccountId) -> Self {
        Self {
            account_id,
            records: vec![],
            worker: worker.clone(),
            account_updates: vec![],
            code_hash_update: None,
        }
    }

    /// Patch and overwrite the info contained inside an [`crate::Account`] in sandbox.
    pub fn account(mut self, account: AccountDetailsPatch) -> Self {
        self.account_updates.push(AccountUpdate::Update(account));
        self
    }

    /// Patch and overwrite the info contained inside an [`crate::Account`] in sandbox. This
    /// will allow us to fetch the current details on the chain and allow us to update
    /// the account details w.r.t to them.
    pub fn account_from_current<F>(mut self, f: F) -> Self
    where
        F: Fn(AccountDetails) -> AccountDetailsPatch + Send + 'static,
    {
        self.account_updates
            .push(AccountUpdate::FromCurrent(Box::new(f)));
        self
    }

    /// Patch the access keys of an account. This will add or overwrite the current access key
    /// contained in sandbox with the access key we specify.
    pub fn access_key(mut self, pk: PublicKey, ak: AccessKey) -> Self {
        self.records.push(StateRecord::AccessKey {
            account_id: self.account_id.clone(),
            public_key: pk.into(),
            access_key: ak.into(),
        });
        self
    }

    /// Patch the access keys of an account. This will add or overwrite the current access keys
    /// contained in sandbox with a list of access keys we specify.
    ///
    /// Similar to [`PatchTransaction::access_key`], but allows us to specify multiple access keys
    pub fn access_keys<I>(mut self, access_keys: I) -> Self
    where
        I: IntoIterator<Item = (PublicKey, AccessKey)>,
    {
        // Move account_id out of self struct so we can appease borrow checker.
        // We'll put it back in after we're done.
        let account_id = self.account_id;

        self.records.extend(
            access_keys
                .into_iter()
                .map(|(pk, ak)| StateRecord::AccessKey {
                    account_id: account_id.clone(),
                    public_key: pk.into(),
                    access_key: ak.into(),
                }),
        );

        self.account_id = account_id;
        self
    }

    /// Sets the code for this account. This will overwrite the current code contained in the account.
    /// Note that if a patch for [`Self::account`] or [`Self::account_from_current`] is specified, the code hash
    /// in those will be overwritten with the code hash of the code we specify here.
    pub fn code(mut self, wasm_bytes: &[u8]) -> Self {
        self.code_hash_update = Some(CryptoHash::hash_bytes(wasm_bytes));
        self.records.push(StateRecord::Contract {
            account_id: self.account_id.clone(),
            code: wasm_bytes.to_vec(),
        });
        self
    }

    /// Patch state into the sandbox network, given a prefix key and value. This will allow us
    /// to set contract state that we have acquired in some manner, where we are able to test
    /// random cases that are hard to come up naturally as state evolves.
    pub fn state(mut self, key: &[u8], value: &[u8]) -> Self {
        self.records.push(StateRecord::Data {
            account_id: self.account_id.clone(),
            data_key: key.to_vec().into(),
            value: value.to_vec().into(),
        });
        self
    }

    /// Patch a series of states into the sandbox network. Similar to [`PatchTransaction::state`],
    /// but allows us to specify multiple state patches at once.
    pub fn states<'b, 'c, I>(mut self, states: I) -> Self
    where
        I: IntoIterator<Item = (&'b [u8], &'c [u8])>,
    {
        // Move account_id out of self struct so we can appease borrow checker.
        // We'll put it back in after we're done.
        let account_id = self.account_id;

        self.records
            .extend(states.into_iter().map(|(key, value)| StateRecord::Data {
                account_id: account_id.clone(),
                data_key: key.to_vec().into(),
                value: value.to_vec().into(),
            }));

        self.account_id = account_id;
        self
    }

    /// Perform the state patch transaction into the sandbox network.
    pub async fn transact(mut self) -> Result<()> {
        // NOTE: updating the account is done here because we need to fetch the current
        // account details from the chain. This is an async operation so it is deferred
        // till the transact function.
        let account_patch = if !self.account_updates.is_empty() {
            let mut account = AccountDetailsPatch::default();
            for update in self.account_updates {
                // reduce the updates into a single account details patch
                account.reduce(match update {
                    AccountUpdate::Update(account) => account,
                    AccountUpdate::FromCurrent(f) => {
                        let account = self.worker.view_account(&self.account_id).await?;
                        f(account)
                    }
                });
            }

            // Update the code hash if the user supplied a code patch.
            if let Some(code_hash) = self.code_hash_update.take() {
                account.code_hash = Some(code_hash);
            }

            Some(account)
        } else if let Some(code_hash) = self.code_hash_update {
            // No account patch, but we have a code patch. We need to fetch the current account
            // to reflect the code hash change.
            let mut account = self.worker.view_account(&self.account_id).await?;
            account.code_hash = code_hash;
            Some(account.into())
        } else {
            None
        };

        // Account patch should be the first entry in the records, since the account might not
        // exist yet and the consequent patches might lookup the account on the chain.
        let records = if let Some(account) = account_patch {
            let account: AccountDetails = account.into();
            let mut records = vec![StateRecord::Account {
                account_id: self.account_id.clone(),
                account: account.into_near_account(),
            }];
            records.extend(self.records);
            records
        } else {
            self.records
        };

        self.worker
            .client()
            .query(&RpcSandboxPatchStateRequest {
                records: records.clone(),
            })
            .await
            .map_err(|err| SandboxErrorCode::PatchStateFailure.custom(err))?;

        self.worker
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|err| SandboxErrorCode::PatchStateFailure.custom(err))?;
        Ok(())
    }
}
