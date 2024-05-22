use std::path::PathBuf;
use std::str::FromStr;

use async_trait::async_trait;
use near_jsonrpc_client::methods::sandbox_fast_forward::RpcSandboxFastForwardRequest;
use near_jsonrpc_client::methods::sandbox_patch_state::RpcSandboxPatchStateRequest;
use near_primitives::state_record::StateRecord;
use near_sandbox_utils as sandbox;

use super::builder::{FromNetworkBuilder, NetworkBuilder};
use super::server::ValidatorKey;
use super::{AllowDevAccountCreation, NetworkClient, NetworkInfo, TopLevelAccountCreator};
use crate::error::SandboxErrorCode;
use crate::network::server::SandboxServer;
use crate::network::Info;
use crate::result::{Execution, ExecutionFinalResult, Result};
use crate::rpc::client::Client;
use crate::types::{AccountId, InMemorySigner, NearToken, SecretKey};
use crate::{Account, Contract, Network, Worker};

// Constant taken from nearcore crate to avoid dependency
const DEFAULT_DEPOSIT: NearToken = NearToken::from_near(100);
/// Local sandboxed environment/network, which can be used to test without interacting with
/// networks that are online such as mainnet and testnet. Look at [`workspaces::sandbox`]
/// for how to spin up a sandboxed network and interact with it.
///
/// [`workspaces::sandbox`]: crate::sandbox
pub struct Sandbox {
    pub(crate) server: SandboxServer,
    client: Client,
    info: Info,
    version: Option<String>,
}

impl Sandbox {
    pub(crate) fn root_signer(&self) -> Result<InMemorySigner> {
        match &self.server.validator_key {
            ValidatorKey::HomeDir(home_dir) => {
                let path = home_dir.join("validator_key.json");
                InMemorySigner::from_file(&path)
            }
            ValidatorKey::Known(account_id, secret_key) => Ok(InMemorySigner::from_secret_key(
                account_id.clone(),
                secret_key.clone(),
            )),
        }
    }
    pub(crate) async fn from_builder_with_version<'a>(
        build: NetworkBuilder<'a, Self>,
        version: &str,
    ) -> Result<Self> {
        // Check the conditions of the provided rpc_url and validator_key
        let mut server = match (build.rpc_addr, build.validator_key) {
            // Connect to a provided sandbox:
            (Some(rpc_url), Some(validator_key)) => {
                SandboxServer::connect(rpc_url, validator_key).await?
            }

            // Spawn a new sandbox since rpc_url and home_dir weren't specified:
            (None, None) => SandboxServer::run_new_with_version(version).await?,

            // Missing inputted parameters for sandbox:
            (Some(rpc_url), None) => {
                return Err(SandboxErrorCode::InitFailure.message(format!(
                    "Custom rpc_url={rpc_url} requires validator_key set."
                )));
            }
            (None, Some(validator_key)) => {
                return Err(SandboxErrorCode::InitFailure.message(format!(
                    "Custom validator_key={validator_key:?} requires rpc_url set."
                )));
            }
        };

        let client = Client::new(&server.rpc_addr(), build.api_key)?;
        client.wait_for_rpc().await?;

        // Server locks some ports on startup due to potential port collision, so we need
        // to unlock the lockfiles after RPC is ready. Not necessarily needed here since
        // they get unlocked anyways on the server's drop, but it is nice to clean up the
        // lockfiles as soon as possible.
        server.unlock_lockfiles()?;

        let info = Info {
            name: build.name.into(),
            root_id: AccountId::from_str("test.near").unwrap(),
            keystore_path: PathBuf::from(".near-credentials/sandbox/"),
            rpc_url: url::Url::parse(&server.rpc_addr()).expect("url is hardcoded"),
        };

        Ok(Self {
            server,
            client,
            info,
            version: Some(version.to_string()),
        })
    }
}

impl std::fmt::Debug for Sandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Sandbox")
            .field("root_id", &self.info.root_id)
            .field("rpc_url", &self.info.rpc_url)
            .field("rpc_port", &self.server.rpc_port())
            .field("net_port", &self.server.net_port())
            .field("version", &self.version)
            .finish()
    }
}

#[async_trait]
impl FromNetworkBuilder for Sandbox {
    async fn from_builder<'a>(build: NetworkBuilder<'a, Self>) -> Result<Self> {
        Self::from_builder_with_version(build, sandbox::DEFAULT_NEAR_SANDBOX_VERSION).await
    }
}

impl AllowDevAccountCreation for Sandbox {}

#[async_trait]
impl TopLevelAccountCreator for Sandbox {
    async fn create_tla(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
    ) -> Result<Execution<Account>> {
        let root_signer = self.root_signer()?;
        let outcome = self
            .client()
            .create_account(&root_signer, &id, sk.public_key(), DEFAULT_DEPOSIT)
            .await?;

        let signer = InMemorySigner::from_secret_key(id, sk);
        Ok(Execution {
            result: Account::new(signer, worker),
            details: ExecutionFinalResult::from_view(outcome),
        })
    }

    async fn create_tla_and_deploy(
        &self,
        worker: Worker<dyn Network>,
        id: AccountId,
        sk: SecretKey,
        wasm: &[u8],
    ) -> Result<Execution<Contract>> {
        let root_signer = self.root_signer()?;
        let outcome = self
            .client()
            .create_account_and_deploy(
                &root_signer,
                &id,
                sk.public_key(),
                DEFAULT_DEPOSIT,
                wasm.into(),
            )
            .await?;

        let signer = InMemorySigner::from_secret_key(id, sk);
        Ok(Execution {
            result: Contract::new(signer, worker),
            details: ExecutionFinalResult::from_view(outcome),
        })
    }
}

impl NetworkClient for Sandbox {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Sandbox {
    fn info(&self) -> &Info {
        &self.info
    }
}

impl Sandbox {
    pub(crate) async fn patch_state(
        &self,
        contract_id: &AccountId,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        let state = StateRecord::Data {
            account_id: contract_id.to_owned(),
            data_key: key.to_vec().into(),
            value: value.to_vec().into(),
        };
        let records = vec![state];

        // NOTE: RpcSandboxPatchStateResponse is an empty struct with no fields, so don't do anything with it:
        let _patch_resp = self
            .client()
            .query(&RpcSandboxPatchStateRequest { records })
            .await
            .map_err(|e| SandboxErrorCode::PatchStateFailure.custom(e))?;

        Ok(())
    }

    pub(crate) async fn fast_forward(&self, delta_height: u64) -> Result<()> {
        // NOTE: RpcSandboxFastForwardResponse is an empty struct with no fields, so don't do anything with it:
        self.client()
            // TODO: replace this with the `query` variant when RpcSandboxFastForwardRequest impls Debug
            .query_nolog(&RpcSandboxFastForwardRequest { delta_height })
            .await
            .map_err(|e| SandboxErrorCode::FastForwardFailure.custom(e))?;

        Ok(())
    }
}
