use alloy::{
    network::AnyNetwork,
    primitives::address,
    providers::{DynProvider, Provider, ProviderBuilder, layers::CacheLayer},
    sol,
};
use ibc_union_spec::{Duration, Timestamp};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use parlia_light_client_types::{ClientState, ClientStateV1, ConsensusState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, instrument};
use unionlabs::{ErrorReporter, ibc::core::client::height::Height, primitives::H160};
use voyager_sdk::{
    anyhow, into_value,
    plugin::ClientBootstrapModule,
    primitives::{ChainId, ClientType},
    rpc::{
        ClientBootstrapModuleServer, FATAL_JSONRPC_ERROR_CODE, types::ClientBootstrapModuleInfo,
    },
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: DynProvider<AnyNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub rpc_url: String,

    #[serde(default)]
    pub max_cache_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientStateConfig {
    // 21 for mainnet, 7 for testnet
    pub valset_size: u64,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ClientBootstrapModuleInfo) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.rpc_url)
                .await?,
        );

        let chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(chain_id.to_string())?;
        info.ensure_client_type(ClientType::PARLIA)?;

        Ok(Self {
            chain_id,
            ibc_handler_address: config.ibc_handler_address,
            provider,
        })
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_client_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        let config = serde_json::from_value::<ClientStateConfig>(config).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "unable to deserialize client state config: {}",
                    ErrorReporter(err)
                ),
                None::<()>,
            )
        })?;

        let valset_epoch_block_number =
            parlia_verifier::calculate_signing_valset_epoch_block_number(
                height.height(),
                config.valset_size,
            );

        info!(%valset_epoch_block_number);

        let rotation_block = self
            .provider
            .get_block(valset_epoch_block_number.into())
            .await
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    ErrorReporter(err).with_message("error fetching initial valset"),
                    None::<()>,
                )
            })?
            .unwrap();

        let (_, initial_valset) = parlia_verifier::parse_epoch_rotation_header_extra_data(
            &rotation_block.header.extra_data,
        )
        .unwrap();

        let unbond_period = Duration::from_secs(
            StakeHub::new(STAKE_HUB_ADDRESS, &self.provider)
                .unbondPeriod()
                .call()
                .await
                .unwrap(),
        );

        info!("unbond period: {unbond_period}");

        Ok(into_value(ClientState::V1(ClientStateV1 {
            latest_height: height.height(),
            chain_id: self
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256; qed;"),
            frozen_height: 0,
            unbond_period,
            ibc_contract_address: self.ibc_handler_address,
            initial_valset: Some(initial_valset),
        })))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_consensus_state(
        &self,
        _: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        let config = serde_json::from_value::<ClientStateConfig>(config).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "unable to deserialize client state config: {}",
                    ErrorReporter(err)
                ),
                None::<()>,
            )
        })?;

        let valset_epoch_block_number =
            parlia_verifier::calculate_signing_valset_epoch_block_number(
                height.height(),
                config.valset_size,
            );

        info!(%valset_epoch_block_number);

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

        Ok(into_value(ConsensusState {
            state_root: block.header.state_root.into(),
            ibc_storage_root: self
                .provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(block.header.number.into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp: Timestamp::from_secs(block.header.timestamp),
            valset_epoch_block_number,
        }))
    }
}

const STAKE_HUB_ADDRESS: alloy::primitives::Address =
    address!("0x0000000000000000000000000000000000002002");

sol! {
    #![sol(rpc)]

    interface StakeHub {
        function unbondPeriod() external view returns (uint64);
    }
}
