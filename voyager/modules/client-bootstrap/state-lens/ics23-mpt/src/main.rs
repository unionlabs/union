use ibc_union_spec::{path::ConsensusStatePath, ClientId, IbcUnion};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state_lens_ics23_mpt_light_client::client::extract_consensus_state;
use state_lens_ics23_mpt_light_client_types::{client_state::Extra, ClientState};
use tracing::{info, instrument};
use unionlabs::{ibc::core::client::height::Height, ErrorReporter};
use voyager_message::{
    into_value,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    primitives::{ChainId, ClientType, QueryHeight},
    ClientBootstrapModule, ExtensionsExt, VoyagerClient, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub l2_chain_id: ChainId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientStateConfig {
    /// The chain that the client will be created on.
    pub host_chain_id: ChainId,
    /// The L1 client on the host chain that tracks the intermediate chain.
    pub l1_client_id: ClientId,
    /// The L2 client on the L1 chain that tracks this L2.
    pub l2_client_id: ClientId,
    #[serde(default = "default_timestamp_offset")]
    pub timestamp_offset: u16,
    #[serde(default = "default_state_root_offset")]
    pub state_root_offset: u16,
    #[serde(default = "default_storage_root_offset")]
    pub storage_root_offset: u16,
}

pub fn default_timestamp_offset() -> u16 {
    0
}

pub fn default_state_root_offset() -> u16 {
    32
}

pub fn default_storage_root_offset() -> u16 {
    64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        Config {}: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        info.ensure_client_type(ClientType::STATE_LENS_ICS23_MPT)?;
        info.ensure_chain_id(info.chain_id.as_str())?;

        Ok(Self {
            l2_chain_id: info.chain_id,
        })
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
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

        Ok(into_value(ClientState {
            l1_client_id: config.l1_client_id,
            l2_chain_id: self.l2_chain_id.to_string(),
            l2_client_id: config.l2_client_id,
            l2_latest_height: height.height(),
            extra: Extra {
                timestamp_offset: config.timestamp_offset,
                state_root_offset: config.state_root_offset,
                storage_root_offset: config.storage_root_offset,
            },
        }))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn self_consensus_state(
        &self,
        ext: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        let config = serde_json::from_value::<ClientStateConfig>(config).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "unable to deserialize consensus state config: {}",
                    ErrorReporter(err)
                ),
                None::<()>,
            )
        })?;

        let voyager_client = ext.try_get::<VoyagerClient>()?;

        let l1_client_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                config.host_chain_id.clone(),
                QueryHeight::Latest,
                config.l1_client_id,
            )
            .await?;

        info!(
            "l1 client {} on host chain {}",
            config.l1_client_id, config.host_chain_id
        );

        let l2_client_info = voyager_client
            .client_info::<IbcUnion>(
                l1_client_meta.counterparty_chain_id.clone(),
                config.l2_client_id,
            )
            .await?;

        let l2_client_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                l1_client_meta.counterparty_chain_id.clone(),
                QueryHeight::Latest,
                config.l2_client_id,
            )
            .await?;

        info!(
            client_type = %l2_client_info.client_type,
            ibc_interface = %l2_client_info.ibc_interface,
            "l2 client info"
        );

        if l2_client_meta.counterparty_chain_id != self.l2_chain_id {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "l2 client {} on {} tracks {}, but this module is for {}",
                    config.l2_client_id,
                    l1_client_meta.counterparty_chain_id,
                    l2_client_meta.counterparty_chain_id,
                    self.l2_chain_id
                ),
                None::<()>,
            ));
        }

        let l1_latest_height = voyager_client
            .query_latest_height(l1_client_meta.counterparty_chain_id.clone(), false)
            .await?;

        info!(
            "l1 latest height ({}) is {l1_latest_height}",
            l1_client_meta.counterparty_chain_id
        );

        let l2_raw_consensus_state = voyager_client
            .query_ibc_state(
                l1_client_meta.counterparty_chain_id.clone(),
                l1_latest_height,
                ConsensusStatePath {
                    client_id: config.l2_client_id,
                    height: height.height(),
                },
            )
            .await?;

        info!(
            "raw consensus state of l2 client {} on l1 chain {}: {l2_raw_consensus_state}",
            config.l2_client_id, l1_client_meta.counterparty_chain_id
        );

        let consensus_state = extract_consensus_state(
            &l2_raw_consensus_state,
            &Extra {
                timestamp_offset: config.timestamp_offset,
                state_root_offset: config.state_root_offset,
                storage_root_offset: config.storage_root_offset,
            },
        );

        info!(
            timestamp = %consensus_state.timestamp,
            state_root = %consensus_state.state_root,
            storage_root = %consensus_state.storage_root,
            "decoded consensus state value of l2 client {} on l1 chain {}",
            config.l2_client_id, l1_client_meta.counterparty_chain_id
        );

        Ok(into_value(&consensus_state))
    }
}
