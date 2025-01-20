use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
};

use alloy::providers::{Provider, ProviderBuilder};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use ogchain_light_client_types::ClientState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tendermint_light_client_types::{ConsensusState, Fraction};
use tracing::{error, instrument};
use unionlabs::{
    ibc::core::{client::height::Height, commitment::merkle_root::MerkleRoot},
    option_unwrap,
    primitives::H160,
    result_unwrap, ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientType},
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    ClientBootstrapModule,
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
    // pub grpc_url: String,
    pub ibc_host_contract_address: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    // pub grpc_url: String,
    pub ibc_host_contract_address: H160,
    pub eth_rpc_url: String,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let tm_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let provider = ProviderBuilder::new()
            .on_builtin(&config.eth_rpc_url)
            .await?;

        let chain_id = provider.get_chain_id().await?;

        info.ensure_chain_id(&chain_id.to_string())?;
        info.ensure_client_type(ClientType::ETHERMINT)?;

        // let chain_revision = chain_id
        //     .split('-')
        //     .last()
        //     .ok_or_else(|| ChainIdParseError {
        //         found: chain_id.clone(),
        //         source: None,
        //     })?
        //     .parse()
        //     .map_err(|err| ChainIdParseError {
        //         found: chain_id.clone(),
        //         source: Some(err),
        //     })?;

        Ok(Self {
            cometbft_client: tm_client,
            chain_id: ChainId::new(chain_id.to_string()),
            // grpc_url: config.grpc_url,
            ibc_host_contract_address: config.ibc_host_contract_address,
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

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        // let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
        //     self.grpc_url.clone(),
        // )
        // .await
        // .unwrap()
        // .params(protos::cosmos::staking::v1beta1::QueryParamsRequest {})
        // .await
        // .unwrap()
        // .into_inner()
        // .params
        // .unwrap();

        let commit = self
            .cometbft_client
            .commit(Some(height.height().try_into().unwrap()))
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        let unbonding_period = std::time::Duration::new(
            1814400,
            0, // params
              //     .unbonding_time
              //     .clone()
              //     .unwrap()
              //     .seconds
              //     .try_into()
              //     .unwrap(),
              // params
              //     .unbonding_time
              //     .clone()
              //     .unwrap()
              //     .nanos
              //     .try_into()
              //     .unwrap(),
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
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            max_clock_drift: const {
                result_unwrap!(unionlabs::google::protobuf::duration::Duration::new(
                    60 * 10,
                    0
                ))
            },
            latest_height: height.inner().try_into().expect("is within bounds; qed;"),
            ibc_contract_address: self.ibc_host_contract_address,
        })
        .unwrap())
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        let commit = self
            .cometbft_client
            .commit(Some(height.height().try_into().unwrap()))
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
                hash: commit.signed_header.header.app_hash.into_encoding(),
            },
            next_validators_hash: commit.signed_header.header.next_validators_hash,
            timestamp: commit.signed_header.header.time,
        })
        .unwrap())
    }
}
