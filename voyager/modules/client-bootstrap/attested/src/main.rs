use alloy::{
    network::AnyNetwork,
    providers::{DynProvider, Provider, ProviderBuilder},
};
use attested_light_client_types::{ClientState, ClientStateV1, ConsensusState};
use ibc_union_spec::Timestamp;
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::{
    anyhow, ensure_null, into_value,
    plugin::ClientBootstrapModule,
    primitives::ChainId,
    rpc::{ClientBootstrapModuleServer, RpcResult, types::ClientBootstrapModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await;
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub provider: DynProvider<AnyNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ClientBootstrapModuleInfo) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = provider.get_chain_id().await?.to_string();

        info.ensure_chain_id(&chain_id)?;

        Ok(Self {
            chain_id: ChainId::new(chain_id),
            provider,
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
            chain_id: self.chain_id.to_string(),
            latest_height: height.height(),
        }))
        .expect("infallible"))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let timestamp = self
            .provider
            .get_block_by_number(height.height().into())
            .await
            .unwrap()
            .unwrap()
            .header
            .timestamp;

        Ok(into_value(ConsensusState {
            timestamp: Timestamp::from_secs(timestamp),
        }))
    }
}
