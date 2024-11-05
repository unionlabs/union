use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use cometbls_light_client_types::{ClientState, ConsensusState};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, instrument};
use unionlabs::{
    ibc::core::{client::height::Height, commitment::merkle_root::MerkleRoot},
    traits::Member,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ConsensusType},
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    run_consensus_module_server, ConsensusModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_consensus_module_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub tm_client: cometbft_rpc::Client,
    pub chain_revision: u64,
    pub grpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ws_url: String,
    pub grpc_url: String,
}

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        info.ensure_chain_id(&chain_id)?;
        info.ensure_consensus_type(ConsensusType::COMETBLS)?;

        let chain_revision = chain_id
            .split('-')
            .last()
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
            tm_client,
            chain_id: ChainId::new(chain_id),
            chain_revision,
            grpc_url: config.grpc_url,
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

    async fn latest_height(&self, finalized: bool) -> Result<Height, cometbft_rpc::JsonRpcError> {
        let commit_response = self.tm_client.commit(None).await?;

        let mut height = commit_response
            .signed_header
            .header
            .height
            .inner()
            .try_into()
            .expect("value is >= 0; qed;");

        if finalized && !commit_response.canonical {
            debug!(
                "commit is not canonical and finalized height was requested, \
                latest finalized height is the previous block"
            );
            height -= 1;
        }

        debug!(height, "latest height");

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
            .map_err(|err| ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>))
    }

    /// Query the latest finalized timestamp of this chain.
    // TODO: Use a better timestamp type here
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query_latest_timestamp(&self, _: &Extensions, finalized: bool) -> RpcResult<i64> {
        let mut commit_response =
            self.tm_client.commit(None).await.map_err(|err| {
                ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>)
            })?;

        if finalized && commit_response.canonical {
            debug!(
                "commit is not canonical and finalized timestamp was \
                requested, fetching commit at previous block"
            );
            commit_response = self
                .tm_client
                .commit(Some(
                    (u64::try_from(commit_response.signed_header.header.height.inner() - 1)
                        .expect("should be fine"))
                    .try_into()
                    .expect("should be fine"),
                ))
                .await
                .map_err(|err| {
                    ErrorObject::owned(-1, ErrorReporter(err).to_string(), None::<()>)
                })?;

            if !commit_response.canonical {
                error!(
                    ?commit_response,
                    "commit for previous height is not canonical? continuing \
                    anyways, but this may cause issues downstream"
                );
            }
        }

        Ok(commit_response
            .signed_header
            .header
            .time
            .as_unix_nanos()
            .try_into()
            .expect("should be fine"))
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .params(protos::cosmos::staking::v1beta1::QueryParamsRequest {})
        .await
        .unwrap()
        .into_inner()
        .params
        .unwrap();

        let commit = self
            .tm_client
            .commit(Some(NonZeroU64::new(height.height()).unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        // Expected to be nanos
        let unbonding_period =
            u64::try_from(params.unbonding_time.clone().unwrap().seconds).unwrap() * 1_000_000_000;

        Ok(serde_json::to_value(ClientState {
            chain_id: cometbls_light_client_types::ChainId::from_string(self.chain_id.to_string())
                .unwrap(),
            trusting_period: unbonding_period * 85 / 100,
            max_clock_drift: (60 * 20) * 1_000_000_000,
            frozen_height: Height::new(0),
            latest_height: Height::new_with_revision(
                self.chain_id
                    .as_str()
                    .split('-')
                    .last()
                    .unwrap()
                    .parse()
                    .unwrap(),
                height.inner().try_into().expect("value is >= 0; qed;"),
            ),
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let commit = self
            .tm_client
            .commit(Some(NonZeroU64::new(height.height()).unwrap()))
            .await
            .unwrap();

        Ok(serde_json::to_value(ConsensusState {
            timestamp: commit.signed_header.header.time.as_unix_nanos(),
            app_hash: MerkleRoot {
                hash: commit.signed_header.header.app_hash.into_encoding(),
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
        })
        .unwrap())
    }
}
