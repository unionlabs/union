use std::collections::VecDeque;

use call::FetchUpdate;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use sui_light_client_types::{checkpoint_summary::CheckpointContents, CertifiedCheckpointSummary};
use sui_sdk::{
    types::{base_types::ObjectID, full_checkpoint_content::CheckpointTransaction},
    SuiClient, SuiClientBuilder,
};
use tracing::instrument;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    call::Call,
    data::{Data, DecodedHeaderMeta, OrderedHeaders},
    hook::UpdateHook,
    module::{PluginInfo, PluginServer, UnexpectedChainIdError},
    primitives::{ChainId, ClientType},
    vm::{data, pass::PassResult, BoxDynError, Op, Visit},
    DefaultCmd, Plugin, PluginMessage, VoyagerMessage,
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
    pub ibc_handler_address: ObjectID,

    pub sui_object_store_rpc_url: String,

    pub sui_client: SuiClient,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
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
            ibc_handler_address: config.ibc_handler_address,
            sui_object_store_rpc_url: config.sui_object_store_rpc_url,
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
    pub ibc_handler_address: ObjectID,

    pub sui_object_store_rpc_url: String,

    /// The RPC endpoint for custom movement apis.
    pub rpc_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyCheckpointData {
    pub checkpoint_summary: CertifiedCheckpointSummary,
    pub checkpoint_contents: CheckpointContents,
    pub transactions: Vec<CheckpointTransaction>,
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
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
                let client = reqwest::Client::new();
                let req = format!("{}/{}.chk", self.sui_object_store_rpc_url, to);
                let res = client.get(req).send().await.unwrap().bytes().await.unwrap();

                let (_, checkpoint) = bcs::from_bytes::<(
                    u8,
                    sui_sdk::types::full_checkpoint_content::CheckpointData,
                )>(&res)
                .unwrap();

                let checkpoint = serde_json::to_string(&checkpoint).unwrap();
                // TODO(aeryz): this is due to some `is_human_readable` thing in somewhere
                // sorry for who reads this code, i'll fix it
                let checkpoint: MyCheckpointData = serde_json::from_str(&checkpoint).unwrap();

                let log = serde_json::to_string(&sui_light_client_types::header::Header {
                    trusted_height: from,
                    checkpoint_summary: checkpoint.checkpoint_summary.data.clone(),
                    sign_info: checkpoint.checkpoint_summary.auth_signature.clone(),
                })
                .unwrap();

                Ok(data(OrderedHeaders {
                    headers: vec![(
                        DecodedHeaderMeta {
                            height: Height::new(to),
                        },
                        serde_json::to_value(sui_light_client_types::header::Header {
                            trusted_height: from,
                            checkpoint_summary: checkpoint.checkpoint_summary.data,
                            sign_info: checkpoint.checkpoint_summary.auth_signature,
                        })
                        .unwrap(),
                    )],
                }))
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
