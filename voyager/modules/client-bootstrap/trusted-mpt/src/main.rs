use alloy::providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use trusted_mpt_light_client_types::{ClientState, ClientStateV1, ConsensusState};
use unionlabs::{ibc::core::client::height::Height, primitives::H160};
use voyager_message::{
    primitives::{ChainId, ClientType},
    ensure_null, into_value,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    ClientBootstrapModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: DynProvider,

    pub whitelisted_relayers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    pub whitelisted_relayers: Vec<String>,

    #[serde(default)]
    pub max_cache_size: u32,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_client_type(ClientType::TRUSTED_MPT)?;

        Ok(Self {
            chain_id,
            ibc_handler_address: config.ibc_handler_address,
            provider,
            whitelisted_relayers: config.whitelisted_relayers,
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
        ensure_null(config)?;

        Ok(serde_json::to_value(ClientState::V1(ClientStateV1 {
            chain_id: self
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256"),
            latest_height: height.height(),
            ibc_contract_address: self.ibc_handler_address,
            whitelisted_relayers: self.whitelisted_relayers.clone(),
        }))
        .expect("infallible"))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let header = self
            .provider
            .get_block_by_number(height.height().into())
            .await
            .unwrap()
            .unwrap()
            .header;

        Ok(into_value(ConsensusState {
            state_root: header.state_root.0.into(),
            storage_root: self
                .provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(height.height().into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp: header.timestamp,
        }))
    }
}
