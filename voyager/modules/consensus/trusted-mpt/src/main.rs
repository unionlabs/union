#![warn(clippy::unwrap_used)]

use alloy::providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{ibc::core::client::height::Height, ErrorReporter};
use voyager_message::{
    core::{ChainId, Timestamp},
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    ConsensusModule,
};
use voyager_vm::BoxDynError;

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

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;

        Ok(Self {
            chain_id,
            provider,
            finality_lag: config.finality_lag,
        })
    }
}

#[async_trait]
impl ConsensusModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_height(&self, _: &Extensions, _finalized: bool) -> RpcResult<Height> {
        self.provider
            .get_block_number()
            .await
            .map(|h| Height::new(h - self.finality_lag))
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, finalized))]
    async fn query_latest_timestamp(
        &self,
        e: &Extensions,
        _finalized: bool,
    ) -> RpcResult<Timestamp> {
        let latest_block_number = self.query_latest_height(e, false).await?;
        let latest_timestamp = self
            .provider
            // .get_block(BlockNumberOrTag::Latest.into())
            .get_block(latest_block_number.height().into())
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
