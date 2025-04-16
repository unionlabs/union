use std::num::ParseIntError;

use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument, trace};
use unionlabs::{
    bech32::Bech32, ibc::core::client::height::Height, primitives::H256, traits::Member,
};
use voyager_message::{
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    primitives::{ChainId, ConsensusType, Timestamp},
    rpc::json_rpc_error_to_error_object,
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

    pub cometbft_client: cometbft_rpc::Client,
    pub chain_revision: u64,

    pub ibc_host_contract_address: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ibc_host_contract_address: Option<Bech32<H256>>,
}

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        info.ensure_chain_id(&chain_id)?;
        info.ensure_consensus_type(ConsensusType::COMETBLS)?;

        let chain_revision = chain_id
            .split('-')
            .next_back()
            .ok_or_else(|| ChainIdParseError {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| ChainIdParseError {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            cometbft_client: tm_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            ibc_host_contract_address: config
                .ibc_host_contract_address
                .map(|a| *a.data())
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`")]
pub struct ChainIdParseError {
    found: String,
    #[source]
    source: Option<ParseIntError>,
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }

    #[instrument(skip_all, fields(%finalized))]
    async fn latest_height(&self, finalized: bool) -> Result<Height, cometbft_rpc::JsonRpcError> {
        let commit_response = self.cometbft_client.commit(None).await?;

        let mut height = commit_response
            .signed_header
            .header
            .height
            .inner()
            .try_into()
            .expect("value is >= 0; qed;");

        if finalized && !commit_response.canonical {
            trace!(
                "commit is not canonical and finalized height was requested, \
                latest finalized height is the previous block"
            );
            height -= 1;
        }

        trace!(height, "latest height");

        Ok(self.make_height(height))
    }
}

#[async_trait]
impl ConsensusModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_height(&self, _: &Extensions, finalized: bool) -> RpcResult<Height> {
        self.latest_height(finalized)
            .await
            // TODO: Add more context here
            .map_err(json_rpc_error_to_error_object)
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(
        &self,
        _: &Extensions,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let mut commit_response = self
            .cometbft_client
            .commit(None)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        if finalized && commit_response.canonical {
            trace!(
                "commit is not canonical and finalized timestamp was \
                requested, fetching commit at previous block"
            );
            commit_response = self
                .cometbft_client
                .commit(Some(
                    (u64::try_from(commit_response.signed_header.header.height.inner() - 1)
                        .expect("should be fine"))
                    .try_into()
                    .expect("should be fine"),
                ))
                .await
                .map_err(json_rpc_error_to_error_object)?;

            if !commit_response.canonical {
                error!(
                    ?commit_response,
                    "commit for previous height is not canonical? continuing \
                    anyways, but this may cause issues downstream"
                );
            }
        }

        Ok(Timestamp::from_nanos(
            commit_response.signed_header.header.time.as_unix_nanos(),
        ))
    }
}
