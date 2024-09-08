use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    convert,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use either::Either;
use futures::{stream::FuturesOrdered, StreamExt, TryFutureExt, TryStreamExt};
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
use tracing::{debug, debug_span, error, instrument, trace, warn, Instrument};
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
    call::{MakeTransactionBatchesWithUpdate, ModuleCall},
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
    pub client: reconnecting_jsonrpc_ws_client::Client,
    pub chain_id: ChainId<'static>,
    pub client_configs: ClientConfigs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId<'static>,
    pub client_configs: BTreeMap<String, ClientConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub min_batch_size: usize,
    pub max_batch_size: usize,
    pub max_wait_time: Duration,
}

#[derive(Debug, Clone)]
pub enum ClientConfigs {
    Any(ClientConfig),
    Many(BTreeMap<ClientId, ClientConfig>),
}

impl ClientConfigs {
    fn new(mut configs: BTreeMap<String, ClientConfig>) -> Self {
        // if-let chains one day
        if configs.len() == 1 && configs.contains_key("*") {
            Self::Any(configs.remove("*").unwrap())
        } else {
            Self::Many(
                configs
                    .into_iter()
                    .map(|x| (x.0.parse().unwrap(), x.1))
                    .collect(),
            )
        }
    }

    fn config_for_client(&self, client_id: &ClientId) -> &ClientConfig {
        match &self {
            ClientConfigs::Any(any) => any,
            ClientConfigs::Many(many) => &many[client_id],
        }
    }

    fn jaq_filter(&self) -> String {
        match self {
            ClientConfigs::Any(_) => "true".to_owned(),
            ClientConfigs::Many(many) => {
                let clients_json = serde_json::to_string(
                    &many.keys().map(|k| (k, ())).collect::<BTreeMap<_, _>>(),
                )
                .unwrap();

                format!("{clients_json} | has($client_id)")
            }
        }
    }
}

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

impl Module {
    fn plugin_name(&self) -> String {
        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config, voyager_socket: String) -> Result<Self, BoxDynError> {
        let client = reconnecting_jsonrpc_ws_client::Client::new({
            let voyager_socket: &'static str = voyager_socket.leak();
            // `self` doesn't exist yet, so use this is copied from `Self::plugin_name`
            let plugin_name = format!("{PLUGIN_NAME}/{}", config.chain_id);
            move || {
                IpcClientBuilder::default()
                    .build(voyager_socket)
                    .instrument(debug_span!("voyager_ipc_client", plugin = %plugin_name))
            }
        })
        .await?;

        // // TODO: Make this a better error
        // assert!(config.min_batch_size <= config.max_batch_size);

        Ok(Self {
            client,
            chain_id: config.chain_id,
            client_configs: ClientConfigs::new(config.client_configs),
        })
    }
}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: None,
            interest_filter: Some(
                format!(
                    r#"
if ."@type" == "data" then
    ."@value" as $data |

    # pull all ibc events that cause an action on this chain (i.e. where we are the destination)
    # the counterparty of the event origin is the destination
    if $data."@type" == "ibc_event" and $data."@value".counterparty_chain_id == "{chain_id}" then
        $data."@value".event."@type" as $event_type |
        $data."@value".event."@value" as $event_data |

        (
            $event_type == "connection_open_init"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "connection_open_try"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "connection_open_ack"
            and ($event_data.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_init"
            and ($event_data.connection.counterparty.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_try"
            and ($event_data.connection.counterparty.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_ack"
            and ($event_data.connection.counterparty.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "send_packet"
            and ($event_data.packet.destination_channel.connection.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "write_acknowledgement"
            and ($event_data.packet.source_channel.connection.client_id as $client_id | {clients_filter})
        ) or ($data."@type" == "plugin"
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
                    clients_filter = self.client_configs.jaq_filter()
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
            ModuleCall::MakeTransactionBatchesWithUpdate(MakeTransactionBatchesWithUpdate {
                client_id,
                batches,
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

                let target_height = batches
                    .iter()
                    .flatten()
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
                            client_id: client_id.clone(),
                            batches,
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
            ModuleCallback::MakeBatchTransaction(cb) => Ok(cb.call(self.chain_id.clone(), datas)),
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

                    let first_seen_at: u64 = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                        .try_into()
                        .expect("how many milliseconds can there be man");

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
                let client_config = &self.client_configs.config_for_client(&client_id);

                events.sort_by_key(|e| e.1.first_seen_at);

                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let is_overdue = |first_seen_at| {
                    Duration::from_millis(first_seen_at) + client_config.max_wait_time < now
                };

                let (mut overdue_events, mut events): (Vec<_>, Vec<_>) =
                    events.into_iter().partition_map(|e| {
                        if is_overdue(e.1.first_seen_at) {
                            Either::Left(e)
                        } else {
                            Either::Right(e)
                        }
                    });

                events.sort_by_key(|e| e.1.provable_height);
                overdue_events.sort_by_key(|e| e.1.provable_height);

                if overdue_events.len() + events.len() < client_config.min_batch_size {
                    warn!(
                        "found {} overdue events and {} non-overdue events, but the min batch \
                        size for this client ({client_id}) is {}",
                        overdue_events.len(),
                        events.len(),
                        client_config.min_batch_size
                    );
                }

                // [...overdue_events_sorted_by_provable_height, ...events_sorted_by_provable_height]
                overdue_events
                    .into_iter()
                    .chain(events)
                    .chunks(client_config.max_batch_size)
                    .into_iter()
                    .map(move |chunk| {
                        let (idxs, events): (Vec<_>, Vec<_>) = chunk.into_iter().unzip();

                        if events.len() == client_config.max_batch_size
                            || events.iter().any(|e| is_overdue(e.first_seen_at))
                        {
                            // this batch is ready to send out, we need to fetch an update for the client on our chain and turn the events into `IbcMessage`s.
                            //
                            // in order to do this, we first need to figure out what height the client is at, and request an update from that height to a height >= the highest height of all of the messages in this batch.
                            // note that we can't request a *specific* height to update to, since not all chains provide this functionality (ethereum being a notable one) - we instead need to wait for the update to be constructed, and then use the new trusted height of the update to fetch our proofs from.
                            //
                            // this will be done in a multi-step aggregation, where first we fetch the update, then construct the messages, and then turn that into a batch transaction.

                            // // the height on the counterparty chain that all of the events are provable at
                            // let target_height = events
                            //     .iter()
                            //     .map(|e| e.provable_height)
                            //     .max()
                            //     .expect("batch has at least one event; qed;");

                            Either::Left((client_id.clone(), (idxs, events)))
                            // self.client
                            //     .client_meta(
                            //         self.chain_id.clone(),
                            //         QueryHeight::Latest,
                            //         client_id.clone(),
                            //     )
                            //     .map_ok({
                            //         let client_id = client_id.clone();
                            //         move |client_meta| {
                            //             (
                            //                 idxs,
                            //                 seq([
                            //                     call(WaitForHeight {
                            //                         chain_id: client_meta.chain_id,
                            //                         height: target_height,
                            //                     }),
                            //                     call(Call::plugin(
                            //                         self.plugin_name(),
                            //                         MakeTransactionBatchWithUpdate {
                            //                             batch: EventBatch { client_id, events },
                            //                         },
                            //                     )),
                            //                 ]),
                            //             )
                            //         }
                            //     }),
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
            .partition_map::<Vec<_>, Vec<_>, _, _, _>(convert::identity);

        let ready = ready
            .into_iter()
            .into_group_map()
            .into_iter()
            .map(|(client_id, events)| {
                // the height on the counterparty chain that all of the events in these batches are provable at
                // we only want to generate one update for all of these batches
                let target_height = events
                    .iter()
                    .flat_map(|x| &x.1)
                    .map(|e| e.provable_height)
                    .max()
                    .expect("batch has at least one event; qed;");

                debug!(%client_id, "querying client meta for client");

                self.client
                    .client_meta(
                        self.chain_id.clone(),
                        QueryHeight::Latest,
                        client_id.clone(),
                    )
                    .map_ok({
                        let client_id = client_id.clone();
                        move |client_meta| {
                            let (idxs, events): (Vec<_>, Vec<_>) = events.into_iter().unzip();

                            (
                                idxs.into_iter().flatten().collect::<Vec<_>>(),
                                seq([
                                    call(WaitForHeight {
                                        chain_id: client_meta.chain_id,
                                        height: target_height,
                                    }),
                                    call(Call::plugin(
                                        self.plugin_name(),
                                        MakeTransactionBatchesWithUpdate {
                                            client_id,
                                            batches: events,
                                        },
                                    )),
                                ]),
                            )
                        }
                    })
            })
            .collect::<FuturesOrdered<_>>();

        Ok(OptimizationResult {
            optimize_further,
            ready: ready
                .map(|x| x)
                .try_collect()
                .await
                .map_err(json_rpc_error_to_rpc_error)?,
        })
    }
}
