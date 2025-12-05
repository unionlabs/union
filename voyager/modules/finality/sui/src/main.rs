use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use sui_sdk::SuiClientBuilder;
use tracing::{debug, trace};
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::{
    anyhow,
    plugin::FinalityModule,
    primitives::{ChainId, ConsensusType, Timestamp},
    rpc::{FinalityModuleServer, RpcError, RpcResult, types::FinalityModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub sui_client: sui_sdk::SuiClient,
}

impl FinalityModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: FinalityModuleInfo) -> anyhow::Result<Self> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        info.ensure_chain_id(&chain_id)?;
        info.ensure_consensus_type(ConsensusType::SUI)?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            sui_client,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The RPC endpoint for sui.
    pub rpc_url: String,
}

impl Module {}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {
    #[error("provider error")]
    Rest(#[from] sui_sdk::error::Error),
}

#[async_trait]
impl FinalityModuleServer for Module {
    async fn query_latest_height(&self, _: &Extensions, _finalized: bool) -> RpcResult<Height> {
        let latest_height = self
            .sui_client
            .read_api()
            .get_latest_checkpoint_sequence_number()
            .await
            .map_err(RpcError::retryable(
                "error fetching latest checkpoint sequence number",
            ))?;

        trace!(latest_height, "latest height");

        Ok(Height::new(latest_height))
    }

    /// Query the latest finalized timestamp of this chain.
    async fn query_latest_timestamp(
        &self,
        e: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let latest_height = self.query_latest_height(e, finalized).await?;

        let checkpoint = self
            .sui_client
            .read_api()
            .get_checkpoint(sui_sdk::rpc_types::CheckpointId::SequenceNumber(
                latest_height.height(),
            ))
            .await
            .map_err(RpcError::retryable(
                "error fetching latest checkpoint sequence number",
            ))?;

        let timestamp = checkpoint.timestamp_ms;

        debug!(%timestamp, %latest_height, "latest timestamp");

        Ok(Timestamp::from_millis(timestamp))
    }
}
