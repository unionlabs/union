use ibc_union_spec::{ClientId, IbcUnion, Timestamp, path::ConsensusStatePath};
use jsonrpsee::{Extensions, core::async_trait};
use proof_lens_light_client_types::{ClientState, ConsensusState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, instrument};
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::{
    ExtensionsExt, VoyagerClient, anyhow, into_value,
    plugin::ClientBootstrapModule,
    primitives::{ChainId, ClientStateMeta, QueryHeight},
    rpc::{ClientBootstrapModuleServer, RpcError, RpcResult, types::ClientBootstrapModuleInfo},
};

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
    pub host_chain_id: ChainId,
    pub l1_client_id: ClientId,
    pub l2_client_id: ClientId,
    pub timestamp_offset: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

impl ClientBootstrapModule for Module {
    type Config = Config;

    async fn new(Config {}: Self::Config, info: ClientBootstrapModuleInfo) -> anyhow::Result<Self> {
        Ok(Self {
            l2_chain_id: info.chain_id,
        })
    }
}

impl Module {
    async fn fetch_and_verify_config(
        &self,
        voyager_client: &VoyagerClient,
        config: &ClientStateConfig,
    ) -> RpcResult<(ClientStateMeta, ClientStateMeta)> {
        let l1_client_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                config.host_chain_id.clone(),
                QueryHeight::Latest,
                config.l1_client_id,
            )
            .await?;

        info!(
            counterparty_height = %l1_client_meta.counterparty_height,
            counterparty_chain_id = %l1_client_meta.counterparty_chain_id,
            "l1 client meta"
        );

        let l2_client_meta = voyager_client
            .client_state_meta::<IbcUnion>(
                l1_client_meta.counterparty_chain_id.clone(),
                QueryHeight::Latest,
                config.l2_client_id,
            )
            .await?;

        info!(
            counterparty_height = %l2_client_meta.counterparty_height,
            counterparty_chain_id = %l2_client_meta.counterparty_chain_id,
            "l2 client meta"
        );

        if l2_client_meta.counterparty_chain_id != self.l2_chain_id {
            return Err(RpcError::fatal_from_message(format!(
                "this module is configured for {}, but {} on {} tracks {}",
                self.l2_chain_id,
                config.l2_client_id,
                l1_client_meta.counterparty_chain_id,
                l2_client_meta.counterparty_chain_id
            )));
        }

        Ok((l1_client_meta, l2_client_meta))
    }
}

fn extract_timestamp(data: &[u8], offset: usize) -> Timestamp {
    Timestamp::from_nanos(u64::from_be_bytes(
        data[offset..offset + 8]
            .try_into()
            .expect("impossible; qed"),
    ))
}

#[async_trait]
impl ClientBootstrapModuleServer for Module {
    #[instrument(skip_all, fields(chain_id = %self.l2_chain_id))]
    async fn self_client_state(
        &self,
        e: &Extensions,
        height: Height,
        config: Value,
    ) -> RpcResult<Value> {
        let config = serde_json::from_value::<ClientStateConfig>(config)
            .map_err(RpcError::fatal("unable to deserialize client state config"))?;

        let (_l1_client_meta, _l2_client_meta) = self
            .fetch_and_verify_config(e.voyager_client()?, &config)
            .await?;

        Ok(into_value(ClientState {
            l1_client_id: config.l1_client_id,
            l2_chain_id: self.l2_chain_id.to_string(),
            l2_client_id: config.l2_client_id,
            l2_latest_height: height.height(),
            timestamp_offset: config.timestamp_offset,
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
        let config = serde_json::from_value::<ClientStateConfig>(config)
            .map_err(RpcError::fatal("unable to deserialize client state config"))?;

        let voyager_client = ext.voyager_client()?;

        let (l1_client_meta, _l2_client_meta) = self
            .fetch_and_verify_config(voyager_client, &config)
            .await?;

        let l2_client_info = voyager_client
            .client_info::<IbcUnion>(
                l1_client_meta.counterparty_chain_id.clone(),
                config.l2_client_id,
            )
            .await?;

        info!(
            client_type = %l2_client_info.client_type,
            ibc_interface = %l2_client_info.ibc_interface,
            "l2 client info"
        );

        let maybe_raw_l2_consensus_state = voyager_client
            .maybe_query_ibc_state(
                l1_client_meta.counterparty_chain_id.clone(),
                QueryHeight::Specific(l1_client_meta.counterparty_height),
                ConsensusStatePath {
                    client_id: config.l2_client_id,
                    height: height.height(),
                },
            )
            .await?
            .state;

        let Some(raw_l2_consensus_state) = maybe_raw_l2_consensus_state else {
            return Err(RpcError::fatal_from_message(format!(
                "there is no consensus state for client {} on {} at height {} (as of l1 height {})",
                config.l2_client_id,
                l1_client_meta.counterparty_chain_id,
                height.height(),
                l1_client_meta.counterparty_height,
            )));
        };

        info!(%raw_l2_consensus_state, "raw l2 consensus state");

        let l2_consensus_state_meta = voyager_client
            .consensus_state_meta::<IbcUnion>(
                l1_client_meta.counterparty_chain_id.clone(),
                QueryHeight::Latest,
                config.l2_client_id,
                height,
            )
            .await?;

        info!(
            timestamp = %l2_consensus_state_meta.timestamp,
            "l2 consensus state meta"
        );

        let extracted_timestamp =
            extract_timestamp(&raw_l2_consensus_state, config.timestamp_offset.into());

        info!(%extracted_timestamp, "extracted timestamp");

        if l2_consensus_state_meta.timestamp != extracted_timestamp {
            return Err(RpcError::fatal_from_message(format!(
                "extracted timestamp {extracted_timestamp} does not match stored timestamp {}",
                l2_consensus_state_meta.timestamp
            )));
        }

        Ok(into_value(&ConsensusState {
            timestamp: l2_consensus_state_meta.timestamp,
            l1_height: l1_client_meta.counterparty_height.height(),
            raw_l2_consensus_state,
        }))
    }
}
