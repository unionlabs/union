use std::collections::VecDeque;

use attested_light_client::msg::QueryMsg;
use attested_light_client_types::Header;
use ibc_union_spec::Timestamp;
use jsonrpsee::{Extensions, core::async_trait};
use protos::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use unionlabs::{
    ibc::core::client::height::Height,
    never::Never,
    primitives::{Bech32, H256},
};
use voyager_sdk::{
    DefaultCmd, anyhow,
    hook::UpdateHook,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::Call,
        data::{Data, DecodedHeaderMeta, OrderedHeaders},
    },
    plugin::Plugin,
    primitives::{ChainId, ClientType},
    rpc::{PluginServer, RpcError, RpcResult, types::PluginInfo},
    vm::{Op, Visit, data, pass::PassResult},
};

use crate::call::{FetchUpdate, ModuleCall};

pub mod call {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum ModuleCall {
        FetchUpdate(FetchUpdate),
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct FetchUpdate {
        pub to: u64,
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await;
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub attestation_client_address: Bech32<H256>,
    pub cometbft_client: cometbft_rpc::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub attestation_client_address: Bech32<H256>,
    pub rpc_url: String,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        Ok(Self {
            chain_id: config.chain_id,
            attestation_client_address: config.attestation_client_address,
            cometbft_client: cometbft_rpc::Client::new(config.rpc_url).await?,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::ATTESTED),
            ),
        }
    }

    async fn cmd(_: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
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
                    UpdateHook::new(
                        &self.chain_id,
                        &ClientType::new(ClientType::ATTESTED),
                        |fetch| {
                            Call::Plugin(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::FetchUpdate(FetchUpdate {
                                    to: fetch.update_to.height(),
                                }),
                            ))
                        },
                    )
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
            ModuleCall::FetchUpdate(FetchUpdate { to }) => {
                let timestamp = self.query_attested_timestamp_at_height(to).await?;

                Ok(data(OrderedHeaders {
                    headers: vec![(
                        DecodedHeaderMeta {
                            height: Height::new(to),
                        },
                        into_value(Header {
                            height: to,
                            timestamp,
                        }),
                    )],
                }))
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }

    async fn query_attested_timestamp_at_height(&self, height: u64) -> RpcResult<Timestamp> {
        let req = QuerySmartContractStateRequest {
            address: self.attestation_client_address.to_string(),
            query_data: serde_json::to_vec(&QueryMsg::TimestampAtHeight {
                chain_id: self.chain_id.to_string(),
                height,
            })
            .unwrap(),
        };

        let raw = self
            .cometbft_client
            .grpc_abci_query::<_, QuerySmartContractStateResponse>(
                "/cosmwasm.wasm.v1.Query/SmartContractState",
                &req,
                None,
                false,
            )
            .await
            .map_err(RpcError::retryable(
                "error fetching attested timestamp at height",
            ))?
            .into_result()
            .map_err(RpcError::retryable(
                "error fetching attested timestamp at height",
            ))?
            .unwrap()
            .data;

        Ok(serde_json::from_slice::<Option<Timestamp>>(&raw)
            .map_err(RpcError::fatal(format_args!(
                "height {height} has not been attested to"
            )))?
            .unwrap())
    }
}
