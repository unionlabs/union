use std::fmt::Debug;

use alloy::providers::{Provider, ProviderBuilder};
use ethereum_light_client_types::ConsensusState as EthConsensusState;
use evm_state_lens_light_client_types::{ClientState, ConsensusState};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    core::{ChainId, ConsensusType, QueryHeight},
    into_value,
    module::{ConsensusModuleInfo, ConsensusModuleServer},
    ConsensusModule, ExtensionsExt, VoyagerClient,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub l1_chain_id: ChainId,
    pub l2_chain_id: ChainId,
    pub l1_client_id: u32,
    pub l2_client_id: u32,
    pub timestamp_offset: u16,
    pub state_root_offset: u16,
    pub storage_root_offset: u16,
    pub l1_tm_client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub l1_client_id: u32,
    pub l2_client_id: u32,
    pub timestamp_offset: u16,
    pub state_root_offset: u16,
    pub storage_root_offset: u16,
    pub l1_comet_ws_url: String,
    pub l2_eth_rpc_url: String,
}

impl ConsensusModule for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError> {
        let l1_tm_client = cometbft_rpc::Client::new(config.l1_comet_ws_url).await?;

        let l1_chain_id = l1_tm_client.status().await?.node_info.network.to_string();

        info.ensure_chain_id(&l1_chain_id)?;
        info.ensure_consensus_type(ConsensusType::COMETBLS)?;

        let provider = ProviderBuilder::new()
            .on_builtin(&config.l2_eth_rpc_url)
            .await?;

        let l2_chain_id = ChainId::new(provider.get_chain_id().await?.to_string());

        info.ensure_chain_id(l2_chain_id.to_string())?;
        info.ensure_consensus_type(ConsensusType::ETHEREUM)?;

        Ok(Self {
            l1_tm_client,
            l1_chain_id: ChainId::new(l1_chain_id),
            l2_chain_id: ChainId::new(l2_chain_id.to_string()),
            l1_client_id: config.l1_client_id,
            l2_client_id: config.l2_client_id,
            timestamp_offset: config.timestamp_offset,
            state_root_offset: config.state_root_offset,
            storage_root_offset: config.storage_root_offset,
        })
    }
}

#[async_trait]
impl ConsensusModuleServer for Module {
    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn query_latest_height(&self, ext: &Extensions, finalized: bool) -> RpcResult<Height> {
        let voy_client = ext.try_get::<VoyagerClient>()?;
        voy_client
            .query_latest_height(self.l2_chain_id.clone(), finalized)
            .await
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn query_latest_timestamp(&self, ext: &Extensions, finalized: bool) -> RpcResult<i64> {
        let voy_client = ext.try_get::<VoyagerClient>()?;
        voy_client
            .query_latest_timestamp(self.l2_chain_id.clone(), finalized)
            .await
    }

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
            .self_consensus_state(self.l2_chain_id.clone(), QueryHeight::Specific(height))
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
