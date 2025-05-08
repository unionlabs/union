use std::collections::VecDeque;

use call::FetchUpdate;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use serde::{Deserialize, Serialize};
use sui_light_client_types::{
    checkpoint_summary::{CheckpointSummary, GasCostSummary},
    crypto::{AuthorityQuorumSignInfo, CryptoBytes, SuiBitmap},
    U64,
};
use sui_sdk::{types::base_types::ObjectID, SuiClient, SuiClientBuilder};
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
            sui_client,
        })
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: UpdateHook::filter(
                &config.chain_id,
                &ClientType::new(ClientType::MOVEMENT),
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

    /// The RPC endpoint for custom movement apis.
    pub rpc_url: String,
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
            ModuleCall::FetchUpdate(FetchUpdate { to, .. }) => {
                // NOTE(aeryz): This only works with Union's custom Movement node. When the following PR is merged,
                // we will uncomment this: https://github.com/movementlabsxyz/movement/pull/645
                // let header = get_lc_header(&self.movement_rest_url, from, to).await;
                Ok(data(OrderedHeaders {
                    headers: vec![(
                        DecodedHeaderMeta {
                            height: Height::new(to),
                        },
                        serde_json::to_value(sui_light_client_types::header::Header {
                            trusted_height: 10,
                            checkpoint_summary: CheckpointSummary {
                                epoch: 10,
                                sequence_number: 10,
                                network_total_transactions: 10,
                                content_digest: sui_light_client_types::digest::Digest(
                                    Default::default(),
                                ),
                                previous_digest: None,
                                epoch_rolling_gas_cost_summary: GasCostSummary {
                                    computation_cost: U64(0),
                                    storage_cost: U64(0),
                                    storage_rebate: U64(0),
                                    non_refundable_storage_fee: U64(0),
                                },
                                timestamp_ms: 10,
                                checkpoint_commitments: vec![],
                                end_of_epoch_data: None,
                                version_specific_data: vec![],
                            },
                            sign_info: AuthorityQuorumSignInfo::<true> {
                                epoch: 10,
                                signature: CryptoBytes(Default::default()),
                                signers_map: SuiBitmap(Default::default()),
                            },
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
