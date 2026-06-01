use std::fmt::Debug;

use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use tracing::{instrument, trace};
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

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub gno_client: gno_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
}

impl FinalityModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: FinalityModuleInfo) -> anyhow::Result<Self> {
        let gno_client = gno_rpc::Client::new(config.rpc_url).await?;

        let chain_id = gno_client.status(None).await?.node_info.network.to_string();

        info.ensure_chain_id(&chain_id)?;
        info.ensure_consensus_type(ConsensusType::GNO)?;

        Ok(Self {
            gno_client,
            chain_id: ChainId::new(chain_id),
        })
    }
}

impl Module {
    #[instrument(skip_all, fields(%finalized))]
    async fn latest_height(&self, finalized: bool) -> RpcResult<Height> {
        let status = self
            .gno_client
            .status(None)
            .await
            .map_err(RpcError::retryable("error fetching status"))?;

        // TODO: Figure out how to check for canonical
        let height = status.sync_info.latest_block_height;

        // if finalized && !commit_response.canonical {
        //     trace!(
        //         "commit is not canonical and finalized height was requested, \
        //         latest finalized height is the previous block"
        //     );
        //     height -= 1;
        // }

        trace!(height, "latest height");

        Ok(Height::new(height))
    }
}

#[async_trait]
impl FinalityModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self, _: &Extensions, finalized: bool) -> RpcResult<Height> {
        self.latest_height(finalized).await
    }

    /// Query the latest finalized timestamp of this chain.
    #[allow(unused)]
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %finalized))]
    async fn query_latest_timestamp(
        &self,
        _: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let status = self.gno_client.status(None).await?;

        // TODO: Figure out how to check for canonical
        // if finalized && !status.canonical {
        //     trace!(
        //         "commit is not canonical and finalized timestamp was \
        //         requested, fetching commit at previous block"
        //     );
        //     status = self
        //         .gno_client
        //         .commit(Some(
        //             (u64::try_from(commit_response.signed_header.header.height.inner() - 1)
        //                 .expect("should be fine"))
        //             .try_into()
        //             .expect("should be fine"),
        //         ))
        //         .await?;

        //     if !commit_response.canonical {
        //         error!(
        //             ?commit_response,
        //             "commit for previous height is not canonical? continuing \
        //             anyways, but this may cause issues downstream"
        //         );
        //     }
        // }

        Ok(Timestamp::from_nanos(
            status.sync_info.latest_block_time.as_unix_nanos(),
        ))
    }
}
