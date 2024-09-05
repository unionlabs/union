use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    convert,
    num::NonZeroUsize,
    ops::RangeInclusive,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use either::Either;
use futures::{stream::FuturesUnordered, TryFutureExt, TryStreamExt};
use itertools::Itertools;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use queue_msg::{
    aggregation::SubsetOf, call, data, now, optimize::OptimizationResult, promise, seq,
    BoxDynError, Op,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, instrument, trace};
use unionlabs::{id::ClientId, QueryHeight};
use voyager_message::{
    call::{Call, FetchUpdateHeaders, WaitForHeight},
    callback::{AggregateMsgUpdateClientsFromOrderedHeaders, Callback},
    data::{ChainEvent, Data, FullIbcEvent},
    default_subcommand_handler,
    plugin::{OptimizationPassPluginServer, PluginInfo, PluginModuleServer},
    reth_ipc::client::IpcClientBuilder,
    rpc::{json_rpc_error_to_rpc_error, VoyagerRpcClient},
    run_module_server, ChainId, PluginMessage, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

use crate::{
    call::{MakeTransactionBatchWithUpdate, ModuleCall},
    callback::{MakeIbcMessagesFromUpdate, ModuleCallback},
    data::{BatchableEvent, EventBatch, ModuleData},
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server(
        Module::new,
        OptimizationPassPluginServer::into_rpc,
        default_subcommand_handler,
    )
    .await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub client: Arc<jsonrpsee::ws_client::WsClient>,
    pub chain_id: ChainId<'static>,
    pub client_configs: BTreeMap<ClientId, ClientConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId<'static>,
    pub client_configs: BTreeMap<ClientId, ClientConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub min_batch_size: NonZeroUsize,
    pub max_batch_size: NonZeroUsize,
    pub max_wait_time: HashMap<ClientId, Duration>,
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config, voyager_socket: String) -> Result<Self, BoxDynError> {
        let client = Arc::new(IpcClientBuilder::default().build(&voyager_socket).await?);

        // // TODO: Make this a better error
        // assert!(config.min_batch_size <= config.max_batch_size);

        Ok(Self {
            client,
            chain_id: config.chain_id,
            client_configs: config.client_configs,
        })
    }
}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument]
    async fn info(&self) -> RpcResult<PluginInfo> {
        let clients_json = serde_json::to_string(
            &self
                .client_configs
                .into_iter()
                .map(|(k, _)| (k, ()))
                .collect::<BTreeMap<_, _>>(),
        )
        .unwrap();

        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: None,
            interest_filter: Some(
                format!(
                    r#"
{clients} as $clients |
if ."@type" == "data" then
    ."@value" as $data |

    # pull all ibc events that cause an action on this chain (i.e. where we are the destination)
    # the counterparty of the event origin is the destination
    if $data."@type" == "ibc_event" and $data."@value".counterparty_chain_id == "{chain_id}" then
        $data."@value".event."@type" as $event_type |
        $data."@value".event."@value" as $event_data |

        ($event_type == "connection_open_init" && ($clients | has($event_data.counterparty_client_id))) or
        ($event_type == "connection_open_try" && ($clients | has($event_data.counterparty_client_id))) or
        ($event_type == "connection_open_ack" && ($clients | has($event_data.counterparty_client_id))) or
        ($event_type == "channel_open_init" && ($clients | has($event_data.connection.counterparty.client_id))) or
        ($event_type == "channel_open_try" && ($clients | has($event_data.connection.counterparty.client_id))) or
        ($event_type == "channel_open_ack" && ($clients | has($event_data.connection.counterparty.client_id))) or
        ($event_type == "send_packet" && ($clients | has($event_data.packet.destination_channel.connection.client_id))) or
        ($event_type == "write_acknowledgement" && ($clients | has($event_data.packet.source_channel.connection.client_id))) or
        ($data."@type" == "plugin"
            and $data."@value".plugin == "{plugin_name}"
            and $data."@value".message."@type" == "event_batch")
    else
        false
    end
else
    false
end
"#,
                    chain_id = self.chain_id,
                    plugin_name = self.plugin_name(),
                    clients = clients_json
                )
                .to_string(),
            ),
        })
    }

    #[instrument]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {
            ModuleCall::MakeMessages(MakeTransactionBatchWithUpdate {
                batch: EventBatch { client_id, events },
            }) => {
                let client_meta = self
                    .client
                    .client_meta(
                        self.chain_id.clone(),
                        QueryHeight::Latest,
                        client_id.clone(),
                    )
                    .await
                    .map_err(json_rpc_error_to_rpc_error)?;

                // let client_info = self
                //     .client
                //     .client_info(self.chain_id.clone(), client_id.clone())
                //     .await
                //     .map_err(json_rpc_error_to_rpc_error)?;

                let latest_height = self
                    .client
                    .query_latest_height(client_meta.chain_id.clone())
                    .await
                    .map_err(json_rpc_error_to_rpc_error)?;

                let target_height = events
                    .iter()
                    .map(|e| e.provable_height)
                    .max()
                    .expect("batch has at least one event; qed;");

                // at this point we assume that a valid update exists - we only ever enqueue this message behind the relevant WaitForHeight on the counterparty chain. to prevent explosions, we do a sanity check here.
                if latest_height < target_height {
                    return Err(ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!(
                            "the latest height of the counterparty chain ({counterparty_chain_id}) \
                            is {latest_height} and the latest trusted height on the client tracking \
                            it ({client_id}) on this chain ({self_chain_id}) is {trusted_height}. \
                            in order to create an update for this client, we need to wait for the \
                            counterparty chain to progress to the next consensus checkpoint greater \
                            than the required target height {target_height}",
                            counterparty_chain_id = client_meta.chain_id,
                            trusted_height = client_meta.height,
                            client_id = client_id,
                            self_chain_id = self.chain_id,
                        ),
                        Some(json!({
                            "current_timestamp": now(),
                        })),
                    ));
                }

                Ok(promise(
                    [promise(
                        [call(FetchUpdateHeaders {
                            counterparty_chain_id: self.chain_id.clone(),
                            chain_id: client_meta.chain_id,
                            update_from: client_meta.height,
                            update_to: latest_height,
                        })],
                        [],
                        AggregateMsgUpdateClientsFromOrderedHeaders {
                            chain_id: self.chain_id.clone(),
                            counterparty_client_id: client_id.clone(),
                        },
                    )],
                    [],
                    Callback::plugin(
                        self.plugin_name(),
                        MakeIbcMessagesFromUpdate {
                            batch: EventBatch {
                                client_id: client_id.clone(),
                                events,
                            },
                        },
                    ),
                ))
            }
        }
    }

    #[instrument]
    async fn callback(
        &self,
        cb: ModuleCallback,
        datas: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {
            ModuleCallback::MakeIbcMessagesFromUpdate(cb) => cb.call(self, datas).await,
            ModuleCallback::MakeBatchTransaction(agg) => {
                Ok(data(agg.call(self.chain_id.clone(), datas)))
            }
        }
    }
}

#[async_trait]
impl OptimizationPassPluginServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        msgs: Vec<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        let mut batchers = HashMap::<ClientId, Vec<(usize, BatchableEvent)>>::new();

        for (idx, msg) in msgs.into_iter().enumerate() {
            let Op::Data(msg) = msg else {
                error!("unexpected message: {msg:?}");

                continue;
            };

            match ChainEvent::try_from_super(msg) {
                Ok(chain_event) => {
                    // the client id of the client on this chain (we are the counterparty from the perspective of the chain where the event was emitted)
                    // this is the client that will need to be updated before this ibc message can be sent
                    let client_id = chain_event
                        .counterparty_client_id()
                        .expect("all batchable messages have a counterparty");

                    trace!(%client_id, "batching event");

                    let first_seen_at = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();

                    batchers.entry(client_id.clone()).or_default().push((
                        idx,
                        match chain_event.event {
                            FullIbcEvent::ConnectionOpenInit(event) => BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::ConnectionOpenTry(event) => BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::ConnectionOpenAck(event) => BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },

                            FullIbcEvent::ChannelOpenInit(event) => BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::ChannelOpenTry(event) => BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::ChannelOpenAck(event) => BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },

                            FullIbcEvent::SendPacket(event) => BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::WriteAcknowledgement(event) => BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },

                            event => panic!("unexpected event: {event:?}"),
                        },
                    ));
                }
                Err(msg) => {
                    match <PluginMessage<EventBatch>>::try_from_super(msg) {
                        Ok(PluginMessage { plugin: _, message }) => {
                            trace!(
                                client_id = %message.client_id,
                                events.len = %message.events.len(),
                                "batching event"
                            );

                            batchers
                                .entry(message.client_id)
                                .or_default()
                                .extend(message.events.into_iter().map(|event| (idx, event)));
                        }
                        Err(msg) => {
                            error!("unexpected message: {msg:?}");
                        }
                    };
                }
            };
        }

        let (ready, optimize_further) = batchers
            .into_iter()
            .flat_map(|(client_id, mut events)| {
                let client_config = self.client_configs[&client_id];

                events.sort_by_key(|e| e.1.first_seen_at);

                events
                    .into_iter()
                    .chunks(self.max_batch_size.get())
                    .into_iter()
                    .map(move |chunk| {
                        let (idxs, events): (Vec<_>, Vec<_>) = chunk.into_iter().unzip();

                        if events.len() == self.max_batch_size.get() {
                            // this batch is ready to send out, we need to fetch an update for the client on our chain and turn the events into `IbcMessage`s.
                            //
                            // in order to do this, we first need to figure out what height the client is at, and request an update from that height to a height >= the highest height of all of the messages in this batch.
                            // note that we can't request a *specific* height to update to, since not all chains provide this functionality (ethereum being a notable one) - we instead need to wait for the update to be constructed, and then use the new trusted height of the update to fetch our proofs from.
                            //
                            // this will be done in a multi-step aggregation, where first we fetch the update, then construct the messages, and then turn that into a batch transaction.

                            // the height on the counterparty chain that all of the events are provable at
                            let target_height = events
                                .iter()
                                .map(|e| e.provable_height)
                                .max()
                                .expect("batch has at least one event; qed;");

                            Either::Left(
                                self.client
                                    .client_meta(
                                        self.chain_id.clone(),
                                        QueryHeight::Latest,
                                        client_id.clone(),
                                    )
                                    .map_ok({
                                        let client_id = client_id.clone();
                                        move |client_meta| {
                                            (
                                                idxs,
                                                seq([
                                                    call(WaitForHeight {
                                                        chain_id: client_meta.chain_id,
                                                        height: target_height,
                                                    }),
                                                    call(Call::plugin(
                                                        self.plugin_name(),
                                                        MakeTransactionBatchWithUpdate {
                                                            batch: EventBatch { client_id, events },
                                                        },
                                                    )),
                                                ]),
                                            )
                                        }
                                    }),
                            )
                        } else {
                            Either::Right((
                                idxs,
                                data(Data::plugin(
                                    self.plugin_name(),
                                    EventBatch {
                                        client_id: client_id.clone(),
                                        events,
                                    },
                                )),
                                self.plugin_name(),
                            ))
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .partition_map::<FuturesUnordered<_>, Vec<_>, _, _, _>(convert::identity);

        Ok(OptimizationResult {
            optimize_further,
            ready: ready
                .try_collect()
                .await
                .map_err(json_rpc_error_to_rpc_error)?,
        })
    }
}
