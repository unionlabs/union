use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use sui_sdk::SuiClientBuilder;
use tracing::{debug, trace};
use unionlabs::{
    aptos::{state_proof::StateProof, transaction_proof::TransactionInfoWithProof},
    ibc::core::client::height::Height,
    ErrorReporter,
};
use voyager_message::{
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    primitives::{ChainId, ConsensusType, Timestamp},
    vm::BoxDynError,
    ConsensusModule,
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

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub sui_client: sui_sdk::SuiClient,
}

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        info.ensure_chain_id(chain_id.to_string())?;
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
    /// The RPC endpoint for aptos.
    pub rpc_url: String,
}

impl Module {}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {
    #[error("provider error")]
    Rest(#[from] sui_sdk::error::Error),
}

#[async_trait]
impl ConsensusModuleServer for Module {
    /// Query the latest finalized height of this chain.
    async fn query_latest_height(&self, _: &Extensions, _finalized: bool) -> RpcResult<Height> {
        match self
            .sui_client
            .read_api()
            .get_latest_checkpoint_sequence_number()
            .await
        {
            Ok(latest_height) => {
                trace!(latest_height, "latest height");
                Ok(Height::new(latest_height))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    /// Query the latest finalized timestamp of this chain.
    async fn query_latest_timestamp(
        &self,
        ext: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let latest_height = self.query_latest_height(ext, finalized).await?;

        match self
            .sui_client
            .read_api()
            .get_checkpoint(sui_sdk::rpc_types::CheckpointId::SequenceNumber(
                latest_height.height(),
            ))
            .await
        {
            Ok(checkpoint) => {
                let timestamp = checkpoint.timestamp_ms;

                debug!(%timestamp, %latest_height, "latest timestamp");

                Ok(Timestamp::from_nanos(timestamp * 1_000_000))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }
}
