use aptos_move_ibc::ibc::ClientExt;
use aptos_rest_client::error::RestError;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use movement_light_client_types::{ClientState, ConsensusState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    aptos::{
        account::AccountAddress, state_proof::StateProof,
        transaction_proof::TransactionInfoWithProof,
    },
    ibc::core::client::height::Height,
    primitives::{FixedBytes, H160, U256},
};
use voyager_message::{
    ensure_null,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    primitives::{ChainId, ClientType, Timestamp},
    vm::BoxDynError,
    ClientBootstrapModule,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StateProofResponse {
    tx_index: u64,
    state_proof: StateProof,
    tx_proof: TransactionInfoWithProof,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    /// The address of the IBC smart contract.
    pub ibc_handler_address: AccountAddress,

    /// The address of the settlement contract on Eth.
    pub l1_settlement_address: H160,

    pub l1_client_id: u32,

    pub aptos_client: aptos_rest_client::Client,

    pub movement_rest_url: String,

    pub whitelisted_relayers: Vec<String>,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let aptos_client = aptos_rest_client::Client::new(config.aptos_rest_api.parse().unwrap());

        let chain_id = aptos_client.get_index().await?.inner().chain_id;

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_client_type(ClientType::MOVEMENT)?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            ibc_handler_address: config.ibc_handler_address,
            aptos_client,
            l1_settlement_address: config.l1_settlement_address,
            l1_client_id: config.l1_client_id,
            movement_rest_url: config.movement_rest_url,
            whitelisted_relayers: config
                .whitelisted_relayers
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}

impl aptos_move_ibc::ibc::ClientExt for Module {
    fn client(&self) -> &aptos_rest_client::Client {
        &self.aptos_client
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: AccountAddress,

    /// The address of the settlement contract on Eth.
    pub l1_settlement_address: H160,

    /// Id of the light client that this client depends on
    pub l1_client_id: u32,

    /// The RPC endpoint for aptos.
    pub aptos_rest_api: String,

    /// The RPC endpoint for custom movement apis.
    pub movement_rest_url: String,

    /// The relayers that are allowed to modify this light client
    ///
    /// Note that the light client had to be permissioned for now since
    /// we are waiting for our [PR] to be merged so that we can fetch
    /// the necessary proofs.
    ///
    /// [PR]: https://github.com/movementlabsxyz/movement/pull/645
    pub whitelisted_relayers: Vec<cosmwasm_std::Addr>,
}

impl Module {
    pub async fn ledger_version_of_height(&self, height: u64) -> u64 {
        let ledger_version = self
            .aptos_client
            .get_block_by_height(height, false)
            .await
            // .map_err(rest_error_to_rpc_error)?
            .unwrap()
            .into_inner()
            .last_version
            .0;

        debug!("height {height} is ledger version {ledger_version}");

        ledger_version
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {
    #[error("provider error")]
    Rest(#[from] RestError),
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let ledger_version = self.ledger_version_of_height(height.height()).await;

        let vault_addr = self
            .get_vault_addr(
                (*self.ibc_handler_address.0.get()).into(),
                Some(ledger_version),
            )
            .await
            .unwrap();

        let table_handle = self
            .aptos_client
            .get_account_resource(
                vault_addr.into(),
                &format!("0x{}::ibc::IBCStore", self.ibc_handler_address),
            )
            .await
            .unwrap()
            .into_inner()
            .unwrap()
            .data["commitments"]["handle"]
            .clone()
            .as_str()
            .unwrap()
            .to_owned();

        Ok(serde_json::to_value(ClientState {
            chain_id: self.chain_id.to_string(),
            l1_client_id: self.l1_client_id,
            l1_contract_address: self.l1_settlement_address,
            l2_contract_address: self.ibc_handler_address,
            table_handle: AccountAddress(FixedBytes::new(
                U256::from_be_hex(table_handle).unwrap().to_be_bytes(),
            )),
            frozen_height: Height::new(0),
            latest_block_num: height.height(),
            whitelisted_relayers: self.whitelisted_relayers.clone(),
        })
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        _height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        Ok(serde_json::to_value(ConsensusState {
            state_root: Default::default(),
            // TODO: ???
            timestamp: Timestamp::from_secs(1000),
            state_proof_hash: Default::default(),
        })
        .expect("infallible"))
    }
}
