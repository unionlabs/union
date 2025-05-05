use std::fmt::Debug;

use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use beacon_api_types::{chain_spec::Mainnet, deneb};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{
    berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX,
    encoding::{DecodeAs, Ssz},
    ibc::core::client::height::Height,
    ErrorReporter,
};
use voyager_message::{
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    primitives::{ChainId, ConsensusType, Timestamp},
    ConsensusModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub comet_chain_id: ChainId,
    pub eth_chain_id: ChainId,
    pub eth_provider: DynProvider,
    pub comet_client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub comet_chain_id: ChainId,
    pub eth_rpc_url: String,
    pub comet_rpc_url: String,
}

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let comet_client = cometbft_rpc::Client::new(config.comet_rpc_url).await?;

        let eth_provider =
            DynProvider::new(ProviderBuilder::new().connect(&config.eth_rpc_url).await?);

        let eth_chain_id = ChainId::new(eth_provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(eth_chain_id.as_str())?;
        info.ensure_consensus_type(ConsensusType::BERACHAIN)?;

        assert_eq!(
            config.comet_chain_id.as_str(),
            comet_client.status().await?.node_info.network
        );

        Ok(Self {
            comet_chain_id: config.comet_chain_id,
            eth_chain_id,
            eth_provider,
            comet_client,
        })
    }
}

#[async_trait]
impl ConsensusModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.eth_chain_id))]
    async fn query_latest_height(&self, _: &Extensions, finalized: bool) -> RpcResult<Height> {
        if finalized {
            let exec_height = self.eth_provider.get_block_number().await.map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error querying latest execution height"),
                    None::<()>,
                )
            })?;

            let raw_execution_header = self
                .comet_client
                .abci_query(
                    "store/beacon/key",
                    [LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX],
                    // proof for height H must be queried at H-1
                    Some((exec_height as i64 - 1).try_into().unwrap()),
                    false,
                )
                .await
                .unwrap();

            let execution_header = deneb::ExecutionPayloadHeaderSsz::<Mainnet>::decode_as::<Ssz>(
                raw_execution_header
                    .response
                    .value
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            -1,
                            format!("execution payload header for block {exec_height} not found"),
                            None::<()>,
                        )
                    })?
                    .as_ref(),
            )
            .unwrap();

            Ok(Height::new(execution_header.block_number))
        } else {
            self.eth_provider
                .get_block_number()
                .await
                .map_err(|e| {
                    ErrorObject::owned(
                        -1,
                        ErrorReporter(e).with_message("error querying latest execution height"),
                        None::<()>,
                    )
                })
                .map(Height::new)
        }
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.eth_chain_id))]
    async fn query_latest_timestamp(
        &self,
        ext: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let latest_height = self.query_latest_height(ext, finalized).await?;
        let latest_block = self
            .eth_provider
            .get_block_by_number(latest_height.height().into())
            .hashes()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error querying latest execution height"),
                    None::<()>,
                )
            })?
            .ok_or_else(|| {
                ErrorObject::owned(-1, format!("block {latest_height} not found"), None::<()>)
            })?;
        // Normalize to nanos in order to be compliant with cosmos
        Ok(Timestamp::from_secs(latest_block.header.timestamp))
    }
}
