// #![warn(clippy::unwrap_used)]

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    network::AnyNetwork,
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{ibc::core::client::height::Height, ErrorReporter};
use voyager_sdk::{
    anyhow,
    plugin::FinalityModule,
    primitives::{ChainId, ConsensusType, Timestamp},
    rpc::{types::FinalityModuleInfo, FinalityModuleServer},
    serde_json::json,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
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
    #[serde(default)]
    pub max_cache_size: u32,
}

impl FinalityModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: FinalityModuleInfo) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .network::<AnyNetwork>()
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_consensus_type(ConsensusType::PARLIA)?;

        Ok(Self { chain_id, provider })
    }
}

#[async_trait]
impl FinalityModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_height(&self, _: &Extensions, finalized: bool) -> RpcResult<Height> {
        let height = self
            .provider
            .get_block(BlockId::Number(if finalized {
                BlockNumberOrTag::Finalized
            } else {
                BlockNumberOrTag::Latest
            }))
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching block for height"),
                    Some(json!({
                        "finalized": finalized,
                    })),
                )
            })?
            .unwrap()
            .header
            .number;

        Ok(Height::new(height))
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_timestamp(
        &self,
        _: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let millis: u64 = self
            .provider
            .get_block(BlockId::Number(if finalized {
                BlockNumberOrTag::Finalized
            } else {
                BlockNumberOrTag::Latest
            }))
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching block for timestamp"),
                    Some(json!({
                        "finalized": finalized,
                    })),
                )
            })?
            .unwrap()
            .other
            .get_deserialized::<alloy::primitives::U64>("milliTimestamp")
            .unwrap()
            .unwrap()
            .into_limbs()[0];

        Ok(Timestamp::from_nanos(millis * 1000_0000))
    }
}
