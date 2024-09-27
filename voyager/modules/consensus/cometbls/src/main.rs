use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use queue_msg::BoxDynError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, instrument};
use unionlabs::{
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        lightclients::cometbls::{client_state::ClientState, consensus_state::ConsensusState},
    },
    traits::Member,
};
use voyager_message::{
    core::{ChainId, ClientType},
    module::{ConsensusModuleInfo, ConsensusModuleServer, ModuleInfo},
    run_module_server, DefaultCmd, ModuleContext,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module>().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,

    pub tm_client: cometbft_rpc::Client,
    pub chain_revision: u64,
    pub grpc_url: String,

    pub prover_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId<'static>,

    pub ws_url: String,
    pub grpc_url: String,

    pub prover_endpoints: Vec<String>,
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = ConsensusModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.ws_url).await?;

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        if chain_id != config.chain_id.as_str() {
            return Err(format!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            )
            .into());
        }

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
            prover_endpoints: config.prover_endpoints,
            grpc_url: config.grpc_url,
        })
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            kind: ConsensusModuleInfo {
                chain_id: config.chain_id,
                client_type: ClientType::new(ClientType::COMETBLS),
            },
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`")]
pub struct ChainIdParseError {
    found: String,
    #[source]
    source: Option<ParseIntError>,
}

#[async_trait]
impl ConsensusModuleServer for Module {
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
            .commit(Some(NonZeroU64::new(height.revision_height).unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        // Expected to be nanos
        let unbonding_period =
            u64::try_from(params.unbonding_time.clone().unwrap().seconds).unwrap() * 1_000_000_000;

        Ok(serde_json::to_value(ClientState {
            chain_id: self.chain_id.to_string(),
            trusting_period: unbonding_period * 85 / 100,
            unbonding_period,
            max_clock_drift: (60 * 20) * 1_000_000_000,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            latest_height: Height {
                revision_number: self
                    .chain_id
                    .as_str()
                    .split('-')
                    .last()
                    .unwrap()
                    .parse()
                    .unwrap(),
                revision_height: height.inner().try_into().expect("value is >= 0; qed;"),
            },
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let commit = self
            .tm_client
            .commit(Some(NonZeroU64::new(height.revision_height).unwrap()))
            .await
            .unwrap();

        Ok(serde_json::to_value(ConsensusState {
            timestamp: commit.signed_header.header.time.as_unix_nanos(),
            app_hash: MerkleRoot {
                hash: commit.signed_header.header.app_hash,
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
        })
        .unwrap())
    }
}
