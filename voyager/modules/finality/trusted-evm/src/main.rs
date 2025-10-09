#![warn(clippy::unwrap_used)]

use alloy::{
    eips::BlockNumberOrTag,
    providers::{DynProvider, Provider, ProviderBuilder, layers::CacheLayer},
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{ErrorReporter, ibc::core::client::height::Height};
use voyager_sdk::{
    anyhow,
    plugin::FinalityModule,
    primitives::{ChainId, ConsensusType, Timestamp},
    rpc::{FinalityModuleServer, types::FinalityModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub finality_lag: u64,

    pub provider: DynProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    /// Consider the `latest_height - finality_lag` to be the latest height
    pub finality_lag: u64,

    #[serde(default)]
    pub max_cache_size: u32,
}

impl FinalityModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: FinalityModuleInfo) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_consensus_type(ConsensusType::TRUSTED_EVM)?;

        Ok(Self {
            chain_id,
            provider,
            finality_lag: config.finality_lag,
        })
    }
}

#[async_trait]
impl FinalityModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_height(&self, _: &Extensions, finalized: bool) -> RpcResult<Height> {
        let lag = if finalized { self.finality_lag } else { 0 };
        self.provider
            .get_block_number()
            .await
            .map(|h| Height::new(h - lag))
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_timestamp(
        &self,
        e: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let block = if finalized {
            self.query_latest_height(e, finalized)
                .await?
                .height()
                .into()
        } else {
            BlockNumberOrTag::Latest
        };
        let latest_timestamp = self
            .provider
            .get_block(block.into())
            .hashes()
            .await
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))?
            .ok_or_else(|| ErrorObject::owned(-1, "latest block not found", None::<()>))?
            .header
            .timestamp;
        // Normalize to nanos in order to be compliant with cosmos
        Ok(Timestamp::from_secs(latest_timestamp))
    }
}
