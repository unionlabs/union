use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
    time::Duration,
};

use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use berachain_light_client_types::{client_state::ClientStateV1, ClientState, ConsensusState};
use ics23::ibc_api::SDK_SPECS;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tendermint_light_client_types::Fraction;
use tracing::{error, instrument};
use unionlabs::{
    ibc::core::{client::height::Height, commitment::merkle_root::MerkleRoot},
    option_unwrap,
    primitives::H160,
    result_unwrap, ErrorReporter,
};
use voyager_message::{
    ensure_null, into_value,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    primitives::{ChainId, ClientType, Timestamp},
    ClientBootstrapModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub comet_chain_id: ChainId,

    pub chain_id: ChainId,

    pub cometbft_client: cometbft_rpc::Client,
    pub chain_revision: u64,

    pub provider: DynProvider,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    /// The RPC endpoint for the consensus chain.
    pub comet_rpc_url: String,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let provider = DynProvider::new(ProviderBuilder::new().connect(&config.rpc_url).await?);

        let chain_id = provider.get_chain_id().await?;

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_client_type(ClientType::BEACON_KIT)?;

        let cometbft_client = cometbft_rpc::Client::new(config.rpc_url).await?;

        let comet_chain_id = cometbft_client
            .status()
            .await?
            .node_info
            .network
            .to_string();

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            provider,
            ibc_handler_address: config.ibc_handler_address,
            chain_revision: chain_id,
            cometbft_client,
            comet_chain_id: ChainId::new(comet_chain_id.to_string()),
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
        Height::new(height)
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let unbonding_period = {
            let params = self
                .cometbft_client
                .grpc_abci_query::<_, protos::cosmos::staking::v1beta1::QueryParamsResponse>(
                    "/cosmos.staking.v1beta1.Query/Params",
                    &protos::cosmos::staking::v1beta1::QueryParamsRequest {},
                    Some(i64::try_from(height.height()).unwrap().try_into().unwrap()),
                    false,
                )
                .await
                .unwrap()
                .value
                .unwrap()
                .params
                .unwrap();

            let unbonding_period = params.unbonding_time.clone().unwrap();

            Duration::new(
                unbonding_period.seconds.try_into().unwrap(),
                unbonding_period.nanos.try_into().unwrap(),
            )
        };

        Ok(into_value(ClientState::V1(ClientStateV1 {
            // l1_client_id: config.l1_client_id,
            chain_id: self.comet_chain_id.as_str().to_string(),
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
            proof_specs: SDK_SPECS.into(),

            evm_chain_id: self.chain_id.as_str().parse().unwrap(),
            latest_height: height.height(),
            ibc_contract_address: self.ibc_handler_address,
        })))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        ensure_null(config)?;

        let block = self
            .provider
            .get_block(height.height().into())
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching block"),
                    None::<()>,
                )
            })?
            .unwrap();

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

        Ok(into_value(ConsensusState {
            comet_timestamp: Timestamp::from_nanos(
                commit.signed_header.header.time.as_unix_nanos(),
            ),
            comet_root: MerkleRoot {
                hash: commit.signed_header.header.app_hash.into_encoding(),
            },
            comet_next_validators_hash: commit.signed_header.header.next_validators_hash,
            evm_state_root: block.header.state_root.into(),
            evm_storage_root: self
                .provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(block.header.number.into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            evm_timestamp: Timestamp::from_secs(block.header.timestamp),
        }))
    }
}
