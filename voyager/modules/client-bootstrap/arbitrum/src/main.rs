use alloy::{
    network::AnyNetwork,
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
};
use arbitrum_light_client_types::{ClientState, ClientStateV1, ConsensusState};
use arbitrum_types::{
    L1_NEXT_NODE_NUM_SLOT, L1_NEXT_NODE_NUM_SLOT_OFFSET_BYTES, L1_NODES_CONFIRM_DATA_OFFSET,
    L1_NODES_SLOT,
};
use ibc_union_spec::{ClientId, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument, warn};
use unionlabs::{
    bounded::BoundedU8, ibc::core::client::height::Height, primitives::H160, result_unwrap,
    ErrorReporter,
};
use voyager_message::{
    core::{ChainId, ClientType, QueryHeight},
    into_value,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    ClientBootstrapModule, ExtensionsExt, VoyagerClient, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub l1_chain_id: ChainId,

    pub union_chain_id: ChainId,

    pub l1_contract_address: H160,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub l1_provider: DynProvider,
    pub l2_provider: DynProvider<AnyNetwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub l1_chain_id: ChainId,

    pub union_chain_id: ChainId,

    pub l1_contract_address: H160,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub l1_rpc_url: String,
    pub l2_rpc_url: String,

    #[serde(default)]
    pub max_cache_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientStateConfig {
    pub l1_client_id: ClientId,
}

impl Module {}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        let l1_provider = DynProvider::new(
            ProviderBuilder::new()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.l1_rpc_url)
                .await?,
        );

        let l2_provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .layer(CacheLayer::new(config.max_cache_size))
                .connect(&config.l2_rpc_url)
                .await?,
        );

        let l1_chain_id = ChainId::new(l1_provider.get_chain_id().await?.to_string());
        let l2_chain_id = ChainId::new(l2_provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(l2_chain_id.to_string())?;
        info.ensure_client_type(ClientType::ARBITRUM)?;

        Ok(Self {
            l1_chain_id: config.l1_chain_id,
            chain_id: l1_chain_id,
            union_chain_id: config.union_chain_id,
            ibc_handler_address: config.ibc_handler_address,
            l1_contract_address: config.l1_contract_address,
            l1_provider,
            l2_provider,
        })
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_client_state(
        &self,
        e: &Extensions,
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

        let voyager_client = e.try_get::<VoyagerClient>()?;

        let l1_client_state_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                self.union_chain_id.clone(),
                QueryHeight::Latest,
                config.l1_client_id,
            )
            .await?;

        debug!(?l1_client_state_meta);

        let l2_height_at_l1_latest_height =
            arbitrum_client::finalized_execution_block_of_l1_height(
                &self.l1_provider,
                &self.l2_provider,
                self.l1_contract_address,
                l1_client_state_meta.counterparty_height.height(),
            )
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(&*e)
                        .with_message("error fetching finalized execution block of l1 height"),
                    None::<()>,
                )
            })?
            .header
            .number;

        warn!("currently this module only supports creating a client at the latest height of the l1 client it tracks. requested the client to be created at {height}, but it will be created at {l2_height_at_l1_latest_height}.");

        Ok(into_value(ClientState::V1(ClientStateV1 {
            l1_client_id: config.l1_client_id,
            latest_height: l2_height_at_l1_latest_height,
            chain_id: self
                .chain_id
                .as_str()
                .parse()
                .expect("self.chain_id is a valid u256; qed;"),
            l1_latest_height: l1_client_state_meta.counterparty_height.height(),
            l1_contract_address: self.l1_contract_address,
            l1_next_node_num_slot: L1_NEXT_NODE_NUM_SLOT.into(),
            l1_nodes_slot: L1_NODES_SLOT.into(),
            l1_next_node_num_slot_offset_bytes: const {
                result_unwrap!(BoundedU8::<0, 24>::new_const(
                    L1_NEXT_NODE_NUM_SLOT_OFFSET_BYTES as u8
                ))
            },
            l1_nodes_confirm_data_offset: L1_NODES_CONFIRM_DATA_OFFSET.into(),
            frozen_height: Height::new(0),
            ibc_contract_address: self.ibc_handler_address,
        })))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.chain_id, %height))]
    async fn self_consensus_state(
        &self,
        e: &Extensions,
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

        let voyager_client = e.try_get::<VoyagerClient>()?;

        let l1_client_state_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                self.union_chain_id.clone(),
                QueryHeight::Latest,
                config.l1_client_id,
            )
            .await?;

        let l2_block_at_l1_latest_height = arbitrum_client::finalized_execution_block_of_l1_height(
            &self.l1_provider,
            &self.l2_provider,
            self.l1_contract_address,
            l1_client_state_meta.counterparty_height.height(),
        )
        .await
        .map_err(|e| {
            ErrorObject::owned(
                -1,
                ErrorReporter(&*e)
                    .with_message("error fetching finalized execution block of l1 height"),
                None::<()>,
            )
        })?;

        warn!("currently this module only supports creating a client at the latest height of the l1 client it tracks. requested the client to be created at {height}, but it will be created at {}.", l2_block_at_l1_latest_height.header.number);

        Ok(into_value(ConsensusState {
            // REVIEW: Add state root?
            ibc_storage_root: self
                .l2_provider
                .get_proof(self.ibc_handler_address.into(), vec![])
                .block_id(l2_block_at_l1_latest_height.header.number.into())
                .await
                .unwrap()
                .storage_hash
                .0
                .into(),
            timestamp: 1_000_000_000 * l2_block_at_l1_latest_height.header.timestamp,
        }))
    }
}
