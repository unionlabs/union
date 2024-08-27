use std::{
    collections::{HashMap, VecDeque},
    convert,
    num::NonZeroUsize,
};

use either::Either;
use frunk::hlist_pat;
use itertools::Itertools;
use jsonrpsee::core::{async_trait, RpcResult};
use queue_msg::{
    aggregation::{do_callback, HListTryFromIterator, SubsetOf},
    call, data,
    optimize::OptimizationResult,
    promise, Op,
};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument, trace, warn};
use unionlabs::{id::ClientId, QueryHeight};
use voyager_message::{
    call::{
        compound::fetch_client_state_meta, FetchClientInfo, MakeMsgAcknowledgement,
        MakeMsgChannelOpenAck, MakeMsgChannelOpenConfirm, MakeMsgChannelOpenTry,
        MakeMsgConnectionOpenAck, MakeMsgConnectionOpenConfirm, MakeMsgConnectionOpenTry,
        MakeMsgRecvPacket,
    },
    callback::{AggregateMsgUpdateClientsFromOrderedHeaders, Callback},
    data::{ChainEvent, Data, DecodedClientStateMeta, FullIbcEvent, OrderedMsgUpdateClients},
    plugin::{OptimizationPassPluginServer, PluginInfo, PluginModuleServer},
    run_module_server, ChainId, PluginMessage, VoyagerMessage,
};

use crate::{
    call::ModuleCall,
    callback::{
        MakeBatchTransaction, MakeIbcMessagesFromUpdate,
        MakeUpdateFromLatestHeightToAtLeastTargetHeight, ModuleCallback,
    },
    data::{BatchableEvent, Event, EventBatch, ModuleData},
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(Module::new, OptimizationPassPluginServer::into_rpc).await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId<'static>,
    pub max_batch_size: NonZeroUsize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId<'static>,
    pub max_batch_size: NonZeroUsize,
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub async fn new(config: Config) -> Result<Self, ModuleInitError> {
        Ok(Self {
            chain_id: config.chain_id,
            max_batch_size: config.max_batch_size,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {}

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

        ($event_type == "send_packet") or
        # ($event_type == "recv_packet") or
        ($event_type == "write_acknowledgement") or
        ($event_type == "channel_open_init") or
        ($event_type == "channel_open_try") or
        ($event_type == "channel_open_ack") or
        ($event_type == "connection_open_init") or
        ($event_type == "connection_open_try") or
        ($event_type == "connection_open_ack") or
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
                    plugin_name = self.plugin_name()
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
        match msg {}
    }

    #[instrument]
    fn callback(
        &self,
        cb: ModuleCallback,
        datas: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {
            ModuleCallback::MakeUpdateFromLatestHeightToAtLeastTargetHeight(aggregate) => {
                Ok(do_callback(aggregate, datas))
            }
            ModuleCallback::MakeIbcMessagesFromUpdate(MakeIbcMessagesFromUpdate { batch }) => {
                let Ok(
                    hlist_pat![
                        updates @ OrderedMsgUpdateClients { .. },
                        client_meta @ DecodedClientStateMeta { .. },
                    ],
                ) = HListTryFromIterator::try_from_iter(datas)
                else {
                    panic!("bad data")
                };

                let new_trusted_height = updates
                    .updates
                    .last()
                    .expect("must have at least one update")
                    .0
                    .height;

                Ok(promise(
                    batch.events.into_iter().map(|batchable_event| {
                        assert!(batchable_event.provable_height <= new_trusted_height);

                        let origin_chain_id = client_meta.state.chain_id.clone();
                        let target_chain_id = self.chain_id.clone();

                        // in this context, we are the destination - the counterparty of the source is the destination
                        match batchable_event.event {
                            Event::ConnectionOpenInit(connection_open_init_event) => {
                                call(MakeMsgConnectionOpenTry {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    connection_open_init_event,
                                })
                            }
                            Event::ConnectionOpenTry(connection_open_try_event) => {
                                call(MakeMsgConnectionOpenAck {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    connection_open_try_event,
                                })
                            }
                            Event::ConnectionOpenAck(connection_open_ack_event) => {
                                call(MakeMsgConnectionOpenConfirm {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    connection_open_ack_event,
                                })
                            }
                            Event::ChannelOpenInit(channel_open_init_event) => {
                                call(MakeMsgChannelOpenTry {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    channel_open_init_event,
                                })
                            }
                            Event::ChannelOpenTry(channel_open_try_event) => {
                                call(MakeMsgChannelOpenAck {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    channel_open_try_event,
                                })
                            }
                            Event::ChannelOpenAck(channel_open_ack_event) => {
                                call(MakeMsgChannelOpenConfirm {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    channel_open_ack_event,
                                })
                            }
                            Event::SendPacket(send_packet_event) => call(MakeMsgRecvPacket {
                                origin_chain_id,
                                origin_chain_proof_height: new_trusted_height,
                                target_chain_id,
                                send_packet_event,
                            }),
                            Event::WriteAcknowledgement(write_acknowledgement_event) => {
                                call(MakeMsgAcknowledgement {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    write_acknowledgement_event,
                                })
                            }
                        }
                    }),
                    [],
                    Callback::plugin(self.plugin_name(), MakeBatchTransaction { updates }),
                ))
            }
            ModuleCallback::MakeBatchTransaction(agg) => {
                Ok(data(agg.do_aggregate(self.chain_id.clone(), datas)))
            }
        }
    }
}

#[async_trait]
impl OptimizationPassPluginServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    fn run_pass(
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

                    batchers.entry(client_id.clone()).or_default().push((
                        idx,
                        match chain_event.event {
                            FullIbcEvent::ConnectionOpenInit(event) => BatchableEvent {
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::ConnectionOpenTry(event) => BatchableEvent {
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::ConnectionOpenAck(event) => BatchableEvent {
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },

                            FullIbcEvent::ChannelOpenInit(event) => BatchableEvent {
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::ChannelOpenTry(event) => BatchableEvent {
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::ChannelOpenAck(event) => BatchableEvent {
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },

                            FullIbcEvent::SendPacket(event) => BatchableEvent {
                                provable_height: chain_event.provable_height,
                                event: event.into(),
                            },
                            FullIbcEvent::WriteAcknowledgement(event) => BatchableEvent {
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

        let (
            ready,
            optimize_further,
        ) = batchers
            .into_iter()
            .flat_map(|(client_id, mut events)| {
                events.sort_by_key(|e| e.1.provable_height);

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

                            Either::Left((
                                idxs,
                                promise(
                                    [
                                        fetch_client_state_meta(
                                            self.chain_id.clone(),
                                            client_id.clone(),
                                            QueryHeight::Latest,
                                        ),
                                        promise(
                                            [
                                                call(FetchClientInfo {
                                                    chain_id: self.chain_id.clone(),
                                                    client_id: client_id.clone(),
                                                }),
                                                // fetch update
                                                promise(
                                                    [fetch_client_state_meta(
                                                        self.chain_id.clone(),
                                                        client_id.clone(),
                                                        QueryHeight::Latest,
                                                    )],
                                                    [],
                                                    Callback::plugin(
                                                        self.plugin_name(),
                                                        MakeUpdateFromLatestHeightToAtLeastTargetHeight {
                                                            target_height,
                                                        },
                                                    ),
                                                ),
                                                // fetch(FetchClientInfo {
                                                //     chain_id: self.chain_id.clone(),
                                                //     client_id: client_id.clone(),
                                                // }),
                                            ],
                                            [],
                                            // make update client messages out of updates
                                            AggregateMsgUpdateClientsFromOrderedHeaders {
                                                counterparty_client_id: client_id.clone(),
                                            },
                                        ),
                                    ],
                                    [],
                                    // make ibc messages out of the events, from the height of the created update
                                    Callback::plugin(
                                        self.plugin_name(),
                                        MakeIbcMessagesFromUpdate {
                                            batch: EventBatch {
                                                client_id: client_id.clone(),
                                                events,
                                            },
                                        },
                                    ),
                                ),
                            ))
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

        Ok(OptimizationResult {
            optimize_further,
            ready,
        })
    }
}
