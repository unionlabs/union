use std::collections::VecDeque;

use call::FetchUpdate;
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sui_light_client_types::{CertifiedCheckpointSummary, checkpoint_summary::CheckpointContents};
use sui_sdk::{
    SuiClient, SuiClientBuilder,
    rpc_types::CheckpointId,
    types::{
        base_types::ObjectID, committee::EpochId, full_checkpoint_content::CheckpointTransaction,
    },
};
use tracing::instrument;
use unionlabs::{ErrorReporter, ibc::core::client::height::Height};
use voyager_sdk::{
    DefaultCmd, anyhow,
    hook::UpdateHook,
    message::{
        PluginMessage, VoyagerMessage,
        call::Call,
        data::{Data, DecodedHeaderMeta, OrderedHeaders},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType},
    rpc::{
        FATAL_JSONRPC_ERROR_CODE, PluginServer,
        types::{PluginInfo, UnexpectedChainIdError},
    },
    vm::{Op, Visit, data, pass::PassResult},
};

use crate::{call::ModuleCall, callback::ModuleCallback};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    /// The address of the IBC smart contract.
    pub ibc_contract: ObjectID,

    pub sui_object_store_rpc_url: String,

    pub sui_client: SuiClient,

    pub graphql_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        if chain_id != config.chain_id.as_str() {
            return Err(UnexpectedChainIdError {
                expected: config.chain_id,
                found: chain_id,
            }
            .into());
        }

        Ok(Self {
            chain_id: config.chain_id,
            ibc_contract: config.ibc_contract,
            sui_object_store_rpc_url: config.sui_object_store_rpc_url,
            graphql_url: config.graphql_url,
            sui_client,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::SUI),
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The identifier of the chain
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_contract: ObjectID,

    pub sui_object_store_rpc_url: String,

    /// The RPC endpoint for custom movement apis.
    pub rpc_url: String,

    pub graphql_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckpointData {
    pub checkpoint_summary: CertifiedCheckpointSummary,
    pub checkpoint_contents: CheckpointContents,
    pub transactions: Vec<CheckpointTransaction>,
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    async fn fetch_epoch_changing_updates(
        &self,
        mut trusted_height: u64,
        from: EpochId,
        to: EpochId,
    ) -> RpcResult<(u64, Vec<(DecodedHeaderMeta, Value)>)> {
        if from == to {
            return Ok((trusted_height, vec![]));
        }

        let client = reqwest::Client::new()
            .post(&self.graphql_url)
            .header("Content-Type", "application/json");

        let mut headers = vec![];

        let mut is_first = true;
        for epoch in from..to {
            let query = json!({
              "query": "query ($epoch_id: UInt53) { epoch(epochId: $epoch_id) { checkpoints(last: 1) { edges { node { sequenceNumber } } }  } }",
              "variables": { "epoch_id": epoch }
            });

            let resp = client
                .try_clone()
                .expect("no body, so this will work")
                .body(query.to_string())
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

            let v: serde_json::Value = serde_json::from_str(&resp).unwrap();

            let update_to = v["data"]["epoch"]["checkpoints"]["edges"][0]["node"]["sequenceNumber"]
                .as_u64()
                .unwrap();

            if is_first && trusted_height == update_to {
                is_first = false;
                continue;
            }

            let checkpoint = self.fetch_checkpoint(update_to).await?;

            headers.push((
                DecodedHeaderMeta {
                    height: Height::new(update_to),
                },
                serde_json::to_value(sui_light_client_types::header::Header {
                    trusted_height,
                    checkpoint_summary: checkpoint.checkpoint_summary.data,
                    sign_info: checkpoint.checkpoint_summary.auth_signature,
                })
                .expect("serde serialization works"),
            ));

            trusted_height = update_to;
        }

        Ok((trusted_height, headers))
    }

    async fn fetch_checkpoint(&self, num: u64) -> RpcResult<CheckpointData> {
        let req = format!("{}/{}.chk", self.sui_object_store_rpc_url, num);
        let client = reqwest::Client::new();
        let res = client
            .get(req)
            .send()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching the checkpoint"),
                    None::<()>,
                )
            })?
            .bytes()
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    ErrorReporter(e).with_message("error fetching the checkpoint"),
                    None::<()>,
                )
            })?;

        let (_, checkpoint) = bcs::from_bytes::<(u8, CheckpointData)>(&res).map_err(|e| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                ErrorReporter(e).with_message("checkpoint data cannot be decoded"),
                None::<()>,
            )
        })?;

        Ok(checkpoint)
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .map(|mut op| {
                    UpdateHook::new(&self.chain_id, &ClientType::new(ClientType::SUI), |fetch| {
                        Call::Plugin(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(FetchUpdate {
                                from: fetch.update_from.height(),
                                to: fetch.update_to.height(),
                            }),
                        ))
                    })
                    .visit_op(&mut op);

                    op
                })
                .enumerate()
                .map(|(i, op)| (vec![i], op))
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::FetchUpdate(FetchUpdate { from, to }) => {
                let from_epoch = self
                    .sui_client
                    .read_api()
                    .get_checkpoint(CheckpointId::SequenceNumber(from))
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            -1,
                            ErrorReporter(e).with_message("error fetching the checkpoint"),
                            None::<()>,
                        )
                    })?
                    .epoch;

                let checkpoint = self.fetch_checkpoint(to).await?;

                let (trusted_height, mut updates) = self
                    .fetch_epoch_changing_updates(
                        from,
                        from_epoch,
                        checkpoint.checkpoint_summary.data.epoch,
                    )
                    .await?;

                updates.push((
                    DecodedHeaderMeta {
                        height: Height::new(to),
                    },
                    serde_json::to_value(sui_light_client_types::header::Header {
                        trusted_height,
                        checkpoint_summary: checkpoint.checkpoint_summary.data,
                        sign_info: checkpoint.checkpoint_summary.auth_signature,
                    })
                    .expect("serde serialization works"),
                ));

                Ok(data(OrderedHeaders { headers: updates }))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: ModuleCallback,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

// pub async fn query_last_checkpoint_of_epoch(config: &Config, epoch_id: u64) -> anyhow::Result<u64> {
//     // GraphQL query to get the last checkpoint of an epoch
//     let query = json!({
//         "query": "query ($epochID: Int) { epoch(id: $epochID) { checkpoints(last: 1) { nodes { sequenceNumber } } } }",
//         "variables": { "epochID": epoch_id }
//     });

//     // Submit the query by POSTing to the GraphQL endpoint
//     let client = reqwest::Client::new();
//     let resp = client
//         .post(config.graphql_url.as_ref().cloned().unwrap())
//         .header("Content-Type", "application/json")
//         .body(query.to_string())
//         .send()
//         .await
//         .expect("Cannot connect to graphql")
//         .text()
//         .await
//         .expect("Cannot parse response");

//     // Parse the JSON response to get the last checkpoint of the epoch
//     let v: Value = serde_json::from_str(resp.as_str()).expect("Incorrect JSON response");
//     let checkpoint_number = v["data"]["epoch"]["checkpoints"]["nodes"][0]["sequenceNumber"]
//         .as_u64()
//         .unwrap();

//     Ok(checkpoint_number)
// }
