use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use ics23::ibc_api::SDK_SPECS;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use queue_msg::BoxDynError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use unionlabs::{
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        lightclients::tendermint::{
            client_state::ClientState, consensus_state::ConsensusState, fraction::Fraction,
        },
    },
    option_unwrap, result_unwrap, ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ConsensusType},
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId<'static>,

    pub ws_url: String,
    pub grpc_url: String,
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
            grpc_url: config.grpc_url,
        })
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            kind: ConsensusModuleInfo {
                chain_id: config.chain_id,
                consensus_type: ConsensusType::new(ConsensusType::TENDERMINT),
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
            .commit(Some(height.revision_height.try_into().unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        let unbonding_period = std::time::Duration::new(
            params
                .unbonding_time
                .clone()
                .unwrap()
                .seconds
                .try_into()
                .unwrap(),
            params
                .unbonding_time
                .clone()
                .unwrap()
                .nanos
                .try_into()
                .unwrap(),
        );

        Ok(serde_json::to_value(ClientState {
            chain_id: self.chain_id.to_string(),
            // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
            trust_level: Fraction {
                numerator: 1,
                denominator: const { option_unwrap!(NonZeroU64::new(3)) },
            },
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            trusting_period: unionlabs::google::protobuf::duration::Duration::new(
                (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
                (unbonding_period * 85 / 100)
                    .subsec_nanos()
                    .try_into()
                    .unwrap(),
            )
            .unwrap(),
            unbonding_period: unionlabs::google::protobuf::duration::Duration::new(
                unbonding_period.as_secs().try_into().unwrap(),
                unbonding_period.subsec_nanos().try_into().unwrap(),
            )
            .unwrap(),
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            max_clock_drift: const {
                result_unwrap!(unionlabs::google::protobuf::duration::Duration::new(
                    60 * 10,
                    0
                ))
            },
            frozen_height: None,
            latest_height: Height {
                revision_number: self.chain_revision,
                revision_height: height.inner().try_into().expect("is within bounds; qed;"),
            },
            proof_specs: SDK_SPECS.into(),
            upgrade_path: vec!["upgrade".into(), "upgradedIBCState".into()],
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let commit = self
            .tm_client
            .commit(Some(height.revision_height.try_into().unwrap()))
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("error fetching commit: {}", ErrorReporter(e)),
                    None::<()>,
                )
            })?;

        Ok(serde_json::to_value(&ConsensusState {
            root: MerkleRoot {
                hash: commit.signed_header.header.app_hash,
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
            timestamp: commit.signed_header.header.time,
        })
        .unwrap())
    }
}
