use ethereum_light_client_types::ConsensusState as EthConsensusState;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state_lens_ics23_mpt_light_client_types::{ClientState, ConsensusState};
use tracing::instrument;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    core::{ChainId, ClientType, QueryHeight},
    into_value,
    module::{ClientBootstrapModuleInfo, ClientBootstrapModuleServer},
    ClientBootstrapModule, ExtensionsExt, VoyagerClient,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub l2_chain_id: ChainId,
    pub l1_client_id: u32,
    pub l2_client_id: u32,
    pub timestamp_offset: u16,
    pub state_root_offset: u16,
    pub storage_root_offset: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub l1_client_id: u32,
    pub l2_client_id: u32,
    pub timestamp_offset: u16,
    pub state_root_offset: u16,
    pub storage_root_offset: u16,
}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError> {
        Ok(Self {
            l2_chain_id: info.chain_id,
            l1_client_id: config.l1_client_id,
            l2_client_id: config.l2_client_id,
            timestamp_offset: config.timestamp_offset,
            state_root_offset: config.state_root_offset,
            storage_root_offset: config.storage_root_offset,
        })
    }
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn self_client_state(&self, _: &Extensions, height: Height) -> RpcResult<Value> {
        Ok(into_value(ClientState {
            l1_client_id: self.l1_client_id,
            l2_chain_id: self.l2_chain_id.to_string(),
            l2_client_id: self.l2_client_id,
            l2_latest_height: height.height(),
            timestamp_offset: self.timestamp_offset,
            state_root_offset: self.state_root_offset,
            storage_root_offset: self.storage_root_offset,
        }))
    }

    /// The consensus state on this chain at the specified `Height`.
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn self_consensus_state(&self, ext: &Extensions, height: Height) -> RpcResult<Value> {
        let voy_client = ext.try_get::<VoyagerClient>()?;
        let state = voy_client
            .self_consensus_state(
                self.l2_chain_id.clone(),
                ClientType::new(ClientType::ETHEREUM),
                QueryHeight::Specific(height),
            )
            .await?
            .state;
        let consensus_state =
            serde_json::from_value::<EthConsensusState>(state).expect("big trouble");
        Ok(into_value(&ConsensusState {
            timestamp: consensus_state.timestamp,
            state_root: consensus_state.state_root,
            storage_root: consensus_state.storage_root,
        }))
    }
}
