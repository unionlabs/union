use alloy::{
    network::AnyNetwork,
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
};
use bob_light_client_types::{ClientState, ClientStateV2, ConsensusState};
use ibc_union_spec::{ClientId, Timestamp};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H160, U256},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow, into_value,
    plugin::ClientBootstrapModule,
    primitives::{ChainId, ClientType},
    rpc::{
        types::ClientBootstrapModuleInfo, ClientBootstrapModuleServer, FATAL_JSONRPC_ERROR_CODE,
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub dispute_game_factory_address: H160,
    pub dispute_game_factory_dispute_game_list_slot: U256,
    pub fault_dispute_game_code_root_claim_index: usize,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: DynProvider<AnyNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub dispute_game_factory_address: H160,
    pub dispute_game_factory_dispute_game_list_slot: U256,
    pub fault_dispute_game_code_root_claim_index: usize,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub rpc_url: String,

    #[serde(default)]
    pub max_cache_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientStateConfig {
    pub l1_client_id: ClientId,
}

impl Module {}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ClientBootstrapModuleInfo) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let l2_chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(l2_chain_id.to_string())?;
        info.ensure_client_type(ClientType::BOB)?;

        Ok(Self {
            chain_id: l2_chain_id,
            provider,
            ibc_handler_address: config.ibc_handler_address,
            dispute_game_factory_address: config.dispute_game_factory_address,
            dispute_game_factory_dispute_game_list_slot: config
                .dispute_game_factory_dispute_game_list_slot,
            fault_dispute_game_code_root_claim_index: config
                .fault_dispute_game_code_root_claim_index,
        })
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        let config = serde_json::from_value::<ClientStateConfig>(config).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "unable to deserialize client state config: {}",
                    ErrorReporter(err)
                ),
                None::<()>,
            )
        })?;

        Ok(into_value(ClientState::V2(ClientStateV2 {
            l1_client_id: config.l1_client_id,
            latest_height: height.height(),
            chain_id: self
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256; qed;"),
            dispute_game_factory_address: self.dispute_game_factory_address,
            dispute_game_factory_dispute_game_list_slot: self
                .dispute_game_factory_dispute_game_list_slot,
            fault_dispute_game_code_root_claim_index: self.fault_dispute_game_code_root_claim_index,
            frozen_height: 0,
            ibc_contract_address: self.ibc_handler_address,
        })))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        _: Value,
    ) -> RpcResult<Value> {
        let l2_block = self
            .provider
            .get_block(height.height().into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching l2 block"),
                    None::<()>,
                )
            })?
            .unwrap();
        Ok(into_value(ConsensusState {
            state_root: l2_block.header.state_root.into(),
            ibc_storage_root: self
                .provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(l2_block.header.number.into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp: Timestamp::from_secs(l2_block.header.timestamp),
        }))
    }
}
