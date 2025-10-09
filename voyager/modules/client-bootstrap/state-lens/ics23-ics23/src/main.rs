use ibc_union_spec::{ClientId, IbcUnion, path::ConsensusStatePath};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use state_lens_ics23_ics23_light_client_types::{ClientState, ConsensusState, client_state::Extra};
use tracing::{info, instrument};
use unionlabs::{
    ErrorReporter,
    ibc::core::client::height::Height,
    primitives::{H256, encoding::Base64},
};
use voyager_sdk::{
    ExtensionsExt, VoyagerClient, anyhow, into_value,
    plugin::ClientBootstrapModule,
    primitives::{ChainId, ClientStateMeta, QueryHeight},
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
    pub l2_chain_id: ChainId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientStateConfig {
    pub host_chain_id: ChainId,
    pub l1_client_id: ClientId,
    pub l2_client_id: ClientId,
    #[serde(flatten)]
    pub extra: Extra,
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
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!(
                    "this module is configured for {}, but {} on {} tracks {}",
                    self.l2_chain_id,
                    config.l2_client_id,
                    l1_client_meta.counterparty_chain_id,
                    l2_client_meta.counterparty_chain_id
                ),
                None::<()>,
            ));
        }

        Ok((l1_client_meta, l2_client_meta))
    }
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

        let (_l1_client_meta, _l2_client_meta) = self
            .fetch_and_verify_config(e.voyager_client()?, &config)
            .await?;

        Ok(into_value(ClientState {
            l1_client_id: config.l1_client_id,
            l2_chain_id: self.l2_chain_id.to_string(),
            l2_client_id: config.l2_client_id,
            l2_latest_height: height.height(),
            extra: config.extra,
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
                    "unable to deserialize client state config: {}",
                    ErrorReporter(err)
                ),
                None::<()>,
            )
        })?;

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

        let state = voyager_client
            .maybe_query_ibc_state(
                l1_client_meta.counterparty_chain_id.clone(),
                QueryHeight::Latest,
                ConsensusStatePath {
                    client_id: config.l2_client_id,
                    height: height.height(),
                },
            )
            .await?
            .state;

        let consensus_state = match state {
            Some(state) => {
                voyager_client
                    .decode_consensus_state::<IbcUnion, Value>(
                        l2_client_info.client_type,
                        l2_client_info.ibc_interface,
                        state,
                    )
                    .await?
            }
            None => {
                return Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!(
                        "there is no consensus state for client {} on {} at height {}",
                        config.l2_client_id,
                        l1_client_meta.counterparty_chain_id,
                        height.height()
                    ),
                    None::<()>,
                ));
            }
        };

        let l2_consensus_state_meta = voyager_client
            .consensus_state_meta::<IbcUnion>(
                l1_client_meta.counterparty_chain_id,
                QueryHeight::Latest,
                config.l2_client_id,
                height,
            )
            .await?;

        info!(
            timestamp = %l2_consensus_state_meta.timestamp,
            "l2 consensus state meta"
        );

        Ok(into_value(&ConsensusState {
            timestamp: l2_consensus_state_meta.timestamp,
            app_hash: serde_json::from_value::<H256<Base64>>(
                consensus_state
                    .pointer("/root/hash")
                    .ok_or_else(|| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            "unable to read /root/hash value of decoded l2 consensus state",
                            Some(json!({
                                "decoded_consensus_state": consensus_state
                            })),
                        )
                    })?
                    .clone(),
            )
            .map_err(|e| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    ErrorReporter(e).with_message(
                        "unable to decode /root/hash value of decoded l2 consensus state",
                    ),
                    Some(json!({
                        "decoded_consensus_state": consensus_state
                    })),
                )
            })?
            .into_encoding(),
        }))
    }
}
