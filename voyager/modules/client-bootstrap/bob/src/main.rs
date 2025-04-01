use alloy::{
    network::AnyNetwork,
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
};
use bob_light_client_types::{ClientState, ClientStateV1, ConsensusState};
use bob_types::L2_OUTPUTS_SLOT;
use ibc_union_spec::ClientId;
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
use voyager_message::{
    core::{ChainId, ClientType},
    into_value,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    ClientBootstrapModule, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub l1_chain_id: ChainId,

    pub union_chain_id: ChainId,

    pub l2_oracle_address: H160,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub l1_provider: DynProvider,
    pub l2_provider: DynProvider<AnyNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub l1_chain_id: ChainId,

    pub union_chain_id: ChainId,

    pub l2_oracle_address: H160,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub l1_rpc_url: String,
    pub l2_rpc_url: String,

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

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let l1_provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.l1_rpc_url)
                .await?,
        );

        let l2_provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.l2_rpc_url)
                .await?,
        );

        let l1_chain_id = ChainId::new(l1_provider.get_chain_id().await?.to_string());
        let l2_chain_id = ChainId::new(l2_provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(l2_chain_id.to_string())?;
        info.ensure_client_type(ClientType::BOB)?;

        Ok(Self {
            l1_chain_id: config.l1_chain_id,
            chain_id: l1_chain_id,
            union_chain_id: config.union_chain_id,
            ibc_handler_address: config.ibc_handler_address,
            l2_oracle_address: config.l2_oracle_address,
            l1_provider,
            l2_provider,
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

        Ok(into_value(ClientState::V1(ClientStateV1 {
            l1_client_id: config.l1_client_id,
            latest_height: height.height(),
            chain_id: self
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256; qed;"),
            l2_oracle_address: self.l2_oracle_address,
            frozen_height: 0,
            ibc_contract_address: self.ibc_handler_address,
            l2_oracle_l2_outputs_slot: U256::from_be_bytes(L2_OUTPUTS_SLOT.to_be_bytes()),
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
            .l2_provider
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
                .l2_provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(l2_block.header.number.into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp: 1_000_000_000 * l2_block.header.timestamp,
        }))
    }
}
