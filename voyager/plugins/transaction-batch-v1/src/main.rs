use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    convert,
    future::Future,
    pin::Pin,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use either::Either;
use futures::{stream::FuturesOrdered, StreamExt, TryFutureExt, TryStreamExt};
use itertools::Itertools;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info, instrument, trace, warn};
use unionlabs::{
    bytes::Bytes,
    ibc::core::{
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{
            self, connection_end::ConnectionEnd, msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    ics24::{ConnectionPath, Path},
    id::{ClientId, ConnectionId},
    DELAY_PERIOD,
};
use voyager_message::{
    call::{FetchUpdateHeaders, WaitForHeight},
    callback::AggregateMsgUpdateClientsFromOrderedHeaders,
    core::{ChainId, QueryHeight},
    data::{ChainEvent, Data, IbcDatagram},
    ibc_v1::{FullIbcEvent, IbcMessage, IbcV1},
    into_value,
    module::{PluginInfo, PluginServer},
    rpc::{json_rpc_error_to_error_object, VoyagerRpcClient},
    DefaultCmd, ExtensionsExt, IbcSpec, Plugin, PluginMessage, RawClientId, VoyagerClient,
    VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, data, now, pass::PassResult, promise, seq, BoxDynError, Op};

use crate::{
    call::{MakeMsgConnectionOpenTry, MakeTransactionBatchesWithUpdate, ModuleCall},
    callback::{make_msgs, MakeIbcMessagesFromUpdate, ModuleCallback},
    data::{BatchableEvent, EventBatch, ModuleData},
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub chain_id: ChainId,
    pub client_configs: ClientConfigs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId,
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

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Module::new(config))
    }

    fn info(config: Self::Config) -> PluginInfo {
        let module = Module::new(config);

        PluginInfo {
            name: module.plugin_name(),
            interest_filter: format!(
                r#"
if ."@type" == "data" then
    ."@value" as $data |

    # pull all ibc events that cause an action on this chain (i.e. where we are the destination)
    # the counterparty of the event origin is the destination
    if $data."@type" == "ibc_event" and $data."@value".counterparty_chain_id == "{chain_id}" and $data."@value".ibc_version_id == "{ibc_version_id}" then
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
                chain_id = module.chain_id,
                plugin_name = module.plugin_name(),
                clients_filter = module.client_configs.jaq_filter(),
                ibc_version_id = IbcV1::ID,
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

impl Module {
    fn plugin_name(&self) -> String {
        format!("{PLUGIN_NAME}/{}", self.chain_id)
    }

    pub fn new(config: Config) -> Self {
        // // TODO: Make this a better error
        // assert!(config.min_batch_size <= config.max_batch_size);

        Self {
            chain_id: config.chain_id,
            client_configs: ClientConfigs::new(config.client_configs),
        }
    }
}

#[async_trait]
impl PluginServer<ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn run_pass(
        &self,
        e: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        self.run_pass_internal(e, msgs).await
    }

    #[instrument(skip_all)]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        let voyager_client = e.try_get::<VoyagerClient>()?;

        match msg {
            ModuleCall::MakeTransactionBatchesWithUpdate(MakeTransactionBatchesWithUpdate {
                client_id,
                batches,
            }) => {
                let client_meta = voyager_client
                    .client_meta(
                        self.chain_id.clone(),
                        IbcV1::ID,
                        QueryHeight::Latest,
                        RawClientId::new(client_id.clone()),
                    )
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // let client_info = self
                //     .client
                //     .client_info(self.chain_id.clone(), client_id.clone())
                //     .await
                //     .map_err(json_rpc_error_to_error_object)?;

                let latest_height = voyager_client
                    .query_latest_height(client_meta.chain_id.clone(), false)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

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

                if client_meta.height >= target_height {
                    info!(
                        "client {client_id} has already been updated to a height \
                        >= the desired target height ({} >= {target_height})",
                        client_meta.height,
                    );

                    make_msgs(
                        self,
                        client_id,
                        batches,
                        None,
                        client_meta.clone(),
                        client_meta.height,
                    )
                } else {
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
                                ibc_version_id: IbcV1::ID,
                                counterparty_client_id: RawClientId::new(client_id.clone()),
                            },
                        )],
                        [],
                        PluginMessage::new(
                            self.plugin_name(),
                            ModuleCallback::from(MakeIbcMessagesFromUpdate {
                                client_id: client_id.clone(),
                                batches,
                            }),
                        ),
                    ))
                }
            }

            ModuleCall::MakeMsgConnectionOpenTry(MakeMsgConnectionOpenTry {
                origin_chain_id,
                origin_chain_proof_height,
                target_chain_id,
                connection_open_init_event,
            }) => {
                let ConnectionHandshakeStateAndProof {
                    connection_state,
                    encoded_connection_state_proof,
                } = mk_connection_handshake_state_and_proofs(
                    voyager_client,
                    origin_chain_id,
                    target_chain_id,
                    connection_open_init_event.client_id.clone(),
                    connection_open_init_event.counterparty_client_id.clone(),
                    connection_open_init_event.connection_id.clone(),
                    origin_chain_proof_height,
                )
                .await?;

                Ok(data(IbcDatagram::new::<IbcV1>(IbcMessage::from(
                    MsgConnectionOpenTry {
                        client_id: connection_open_init_event.counterparty_client_id,
                        counterparty: connection::counterparty::Counterparty {
                            client_id: connection_open_init_event.client_id,
                            connection_id: Some(connection_open_init_event.connection_id),
                            prefix: MerklePrefix {
                                // TODO: Make configurable
                                key_prefix: b"ibc".to_vec(),
                            },
                        },
                        // TODO: Make configurable
                        delay_period: DELAY_PERIOD,
                        counterparty_versions: connection_state.versions,
                        proof_height: origin_chain_proof_height,
                        proof_init: encoded_connection_state_proof,
                    },
                ))))
            }

            // ModuleCall::MakeMsgConnectionOpenAck(MakeMsgConnectionOpenAck {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     connection_open_try_event,
            // }) => {
            //     let ConnectionHandshakeStateAndProofs {
            //         connection_state,
            //         encoded_connection_state_proof,
            //         consensus_height,
            //     } = mk_connection_handshake_state_and_proofs(
            //         &voyager_client,
            //         origin_chain_id,
            //         target_chain_id,
            //         connection_open_try_event.client_id,
            //         connection_open_try_event.counterparty_client_id,
            //         connection_open_try_event.connection_id.clone(),
            //         origin_chain_proof_height,
            //     )
            //     .await?;

            //     Ok(voyager_vm::data(IbcMessage::from(MsgConnectionOpenAck {
            //         connection_id: connection_open_try_event.counterparty_connection_id,
            //         counterparty_connection_id: connection_open_try_event.connection_id,
            //         client_state: encoded_client_state,
            //         version: connection_state.versions[0].clone(),
            //         proof_height: origin_chain_proof_height,
            //         proof_try: encoded_connection_state_proof,
            //         proof_client: encoded_client_state_proof,
            //         proof_consensus: encoded_consensus_state_proof,
            //         consensus_height,
            //     })))
            // }

            // ModuleCall::MakeMsgConnectionOpenConfirm(MakeMsgConnectionOpenConfirm {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     connection_open_ack_event,
            // }) => {
            //     // info of the client on the target chain that will verify the storage
            //     // proofs
            //     let target_client_info = &voyager_client
            //         .rpc_server
            //         // counterparty_client_id from open_try is the client on the target chain
            //         .client_info(
            //             &target_chain_id,
            //             connection_open_ack_event.counterparty_client_id.clone(),
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     // proof of connection_state, encoded for the client on the target chain
            //     // this is encoded via the client module for the client on the origin chain
            //     // (the chain the event was emitted on)
            //     let connection_proof = &voyager_client
            //         .rpc_server
            //         .encode_proof(
            //             &target_client_info.client_type,
            //             &target_client_info.ibc_interface,
            //             &voyager_client
            //                 .rpc_server
            //                 .query_ibc_proof(
            //                     &origin_chain_id,
            //                     origin_chain_proof_height,
            //                     ConnectionPath {
            //                         connection_id: connection_open_ack_event.connection_id.clone(),
            //                     }
            //                     .into(),
            //                 )
            //                 .await
            //                 .map_err(error_object_to_queue_error)?
            //                 .proof,
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     Ok(voyager_vm::data(IbcMessage::from(
            //         MsgConnectionOpenConfirm {
            //             connection_id: connection_open_ack_event.counterparty_connection_id,
            //             proof_height: origin_chain_proof_height,
            //             proof_ack: connection_proof,
            //         },
            //     )))
            // }

            // ModuleCall::MakeMsgChannelOpenTry(MakeMsgChannelOpenTry {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     channel_open_init_event: event,
            // }) => {
            //     let origin_channel = voyager_client
            //         .query_channel(
            //             origin_chain_id.clone(),
            //             QueryHeight::Specific(origin_chain_proof_height),
            //             event.port_id.clone(),
            //             event.channel_id.clone(),
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     let proof_init = voyager_client
            //         .query_ibc_proof(
            //             &origin_chain_id,
            //             origin_chain_proof_height,
            //             ChannelEndPath {
            //                 port_id: event.port_id.clone(),
            //                 channel_id: event.channel_id.clone(),
            //             }
            //             .into(),
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     let client_info = voyager_client
            //         .client_info(&target_chain_id, event.connection.counterparty.client_id)
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     let encoded_proof_init = voyager_client
            //         .encode_proof(
            //             &client_info.client_type,
            //             &client_info.ibc_interface,
            //             proof_init.proof,
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     Ok(data(IbcMessage::from(MsgChannelOpenTry {
            //         port_id: event.counterparty_port_id,
            //         channel: Channel {
            //             state: channel::state::State::Tryopen,
            //             ordering: origin_channel
            //                 .state
            //                 .ok_or(QueueError::Fatal("channel must exist".into()))?
            //                 .ordering,
            //             counterparty: channel::counterparty::Counterparty {
            //                 port_id: event.port_id,
            //                 channel_id: Some(event.channel_id),
            //             },
            //             connection_hops: vec![event.connection.counterparty.connection_id.unwrap()],
            //             version: event.version.clone(),
            //             upgrade_sequence: 0,
            //         },
            //         counterparty_version: event.version,
            //         proof_init: encoded_proof_init,
            //         proof_height: origin_chain_proof_height,
            //     })))
            // }

            // ModuleCall::MakeMsgChannelOpenAck(MakeMsgChannelOpenAck {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     channel_open_try_event,
            // }) => {
            //     let origin_channel_path = ChannelEndPath {
            //         port_id: channel_open_try_event.port_id.clone(),
            //         channel_id: channel_open_try_event.channel_id.clone(),
            //     };

            //     let proof_try = voyager_client
            //         .query_ibc_proof(
            //             &origin_chain_id,
            //             origin_chain_proof_height,
            //             origin_channel_path.into(),
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     let client_info = voyager_client
            //         .client_info(
            //             &target_chain_id,
            //             channel_open_try_event.connection.counterparty.client_id,
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     let encoded_proof_try = voyager_client
            //         .encode_proof(
            //             &client_info.client_type,
            //             &client_info.ibc_interface,
            //             proof_try.proof,
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     Ok(data(IbcMessage::from(MsgChannelOpenAck {
            //         port_id: channel_open_try_event.counterparty_port_id,
            //         channel_id: channel_open_try_event.counterparty_channel_id,
            //         counterparty_channel_id: channel_open_try_event.channel_id,
            //         counterparty_version: channel_open_try_event.version,
            //         proof_try: encoded_proof_try,
            //         proof_height: origin_chain_proof_height,
            //     })))
            // }

            // ModuleCall::MakeMsgChannelOpenConfirm(MakeMsgChannelOpenConfirm {
            //     origin_chain_id,
            //     origin_chain_proof_height,
            //     target_chain_id,
            //     channel_open_ack_event,
            // }) => {
            //     let origin_channel_path = ChannelEndPath {
            //         port_id: channel_open_ack_event.port_id.clone(),
            //         channel_id: channel_open_ack_event.channel_id.clone(),
            //     };

            //     let proof_ack = voyager_client
            //         .query_ibc_proof(
            //             &origin_chain_id,
            //             origin_chain_proof_height,
            //             origin_channel_path.into(),
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     let client_info = voyager_client
            //         .client_info(
            //             &target_chain_id,
            //             channel_open_ack_event.connection.counterparty.client_id,
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     let encoded_proof_ack = voyager_client
            //         .encode_proof(
            //             &client_info.client_type,
            //             &client_info.ibc_interface,
            //             proof_ack.proof,
            //         )
            //         .await
            //         .map_err(error_object_to_queue_error)?;

            //     Ok(voyager_vm::data(IbcMessage::from(MsgChannelOpenConfirm {
            //         port_id: channel_open_ack_event.counterparty_port_id,
            //         channel_id: channel_open_ack_event.counterparty_channel_id,
            //         proof_ack: encoded_proof_ack,
            //         proof_height: origin_chain_proof_height,
            //     })))
            // }

            // ModuleCall::MakeMsgRecvPacket(msg) => make_msg_recv_packet(ctx, msg).await,
            _ => todo!(),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        e: &Extensions,
        cb: ModuleCallback,
        datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {
            ModuleCallback::MakeIbcMessagesFromUpdate(cb) => {
                cb.call(e.try_get()?, self, datas).await
            }
            ModuleCallback::MakeBatchTransaction(cb) => Ok(cb.call(self.chain_id.clone(), datas)),
        }
    }
}

impl Module {
    #[allow(clippy::type_complexity)] // if you knew why this was here you'd leave me alone
    fn run_pass_internal<'a>(
        &'a self,
        e: &'a Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> Pin<Box<dyn Future<Output = RpcResult<PassResult<VoyagerMessage>>> + Send + 'a>> {
        Box::pin(async move {
            let mut batchers = HashMap::<ClientId, Vec<(usize, BatchableEvent)>>::new();

            for (idx, msg) in msgs.into_iter().enumerate() {
                let Op::Data(msg) = msg else {
                    error!("unexpected message: {msg:?}");

                    continue;
                };

                match ChainEvent::try_from(msg) {
                    Ok(chain_event) => {
                        // the client id of the client on this chain (we are the counterparty from the perspective of the chain where the event was emitted)
                        // this is the client that will need to be updated before this ibc message can be sent
                        let full_ibc_event = chain_event.decode_event::<IbcV1>().unwrap().unwrap();

                        let client_id = full_ibc_event
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
                            match full_ibc_event {
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
                        match msg.as_plugin::<ModuleData>(self.plugin_name()) {
                            Ok(ModuleData::BatchEvents(message)) => {
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

                    if !overdue_events.is_empty()
                        && overdue_events.len() + events.len() < client_config.min_batch_size
                    {
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
                                Either::Left((client_id.clone(), (idxs, events)))
                            } else {
                                Either::Right((
                                    idxs,
                                    data(PluginMessage::new(
                                        self.plugin_name(),
                                        ModuleData::from(EventBatch {
                                            client_id: client_id.clone(),
                                            events,
                                        }),
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
                .map(|(client_id, events)| async move {
                    let client = e.try_get::<VoyagerClient>()?;

                    // let client_meta =

                    // the height on the counterparty chain that all of the events in these batches are provable at
                    // we only want to generate one update for all of these batches
                    let target_height = events
                        .iter()
                        .flat_map(|x| &x.1)
                        .map(|e| e.provable_height)
                        .max()
                        .expect("batch has at least one event; qed;");

                    debug!(%client_id, "querying client meta for client");

                    client
                        .client_meta(
                            self.chain_id.clone(),
                            IbcV1::ID,
                            QueryHeight::Latest,
                            RawClientId::new(client_id.clone()),
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
                                            finalized: true,
                                        }),
                                        call(PluginMessage::new(
                                            self.plugin_name(),
                                            ModuleCall::from(MakeTransactionBatchesWithUpdate {
                                                client_id,
                                                batches: events,
                                            }),
                                        )),
                                    ]),
                                )
                            }
                        })
                        .await
                })
                .collect::<FuturesOrdered<_>>();

            Ok(PassResult {
                optimize_further,
                ready: ready
                    .map(|x| x)
                    .try_collect()
                    .await
                    .map_err(json_rpc_error_to_error_object)?,
            })
        })
    }
}

/// Used to fetch and construct the state and proofs for
/// MsgConnectionOpenTry/Ack.
#[instrument(
    skip_all,
    fields(
        %origin_chain_id,
        %target_chain_id,
        %client_id,
        %counterparty_client_id,
        %connection_id,
        %origin_chain_proof_height,
    )
)]
async fn mk_connection_handshake_state_and_proofs(
    voyager_client: &VoyagerClient,
    origin_chain_id: ChainId,
    target_chain_id: ChainId,
    client_id: ClientId,
    counterparty_client_id: ClientId,
    connection_id: ConnectionId,
    origin_chain_proof_height: Height,
) -> RpcResult<ConnectionHandshakeStateAndProof> {
    // info of the client on the target chain that will verify the storage
    // proofs
    let target_client_info = voyager_client
        // counterparty_client_id from open_init/try is the client on the target chain
        .client_info(
            target_chain_id.clone(),
            IbcV1::ID,
            RawClientId::new(counterparty_client_id.clone()),
        )
        .await
        .map_err(json_rpc_error_to_error_object)?;

    debug!(
        %counterparty_client_id,
        %target_client_info.client_type,
        %target_client_info.ibc_interface,
        %target_client_info.metadata,
    );

    // info of the client on the origin chain, this is used to decode the stored
    // client state
    let origin_client_info = voyager_client
        // client_id from open_init/try is the client on the origin chain
        .client_info(
            origin_chain_id.clone(),
            IbcV1::ID,
            RawClientId::new(client_id.clone()),
        )
        .await
        .map_err(json_rpc_error_to_error_object)?;

    debug!(
        %client_id,
        %origin_client_info.client_type,
        %origin_client_info.ibc_interface,
        %origin_client_info.metadata,
    );

    // the connection end as stored by the origin chain after open_init/try
    let connection_state = voyager_client
        .query_spec_ibc_state(
            origin_chain_id.clone(),
            origin_chain_proof_height.into(),
            ConnectionPath {
                connection_id: connection_id.clone(),
            },
        )
        .await?
        .state
        .ok_or(ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            "connection must exist",
            None::<()>,
        ))?;
    debug!(
        connection_state = %serde_json::to_string(&connection_state).unwrap(),
    );

    // proof of connection_state, encoded for the client on the target chain
    let connection_proof = voyager_client
        .query_ibc_proof(
            origin_chain_id.clone(),
            IbcV1::ID,
            QueryHeight::Specific(origin_chain_proof_height),
            into_value(Path::from(ConnectionPath {
                connection_id: connection_id.clone(),
            })),
        )
        .await
        .map_err(json_rpc_error_to_error_object)?
        .proof;
    debug!(%connection_proof);

    let encoded_connection_state_proof = voyager_client
        .encode_proof(
            target_client_info.client_type.clone(),
            target_client_info.ibc_interface.clone(),
            connection_proof,
        )
        .await
        .map_err(json_rpc_error_to_error_object)?;
    debug!(%encoded_connection_state_proof);

    Ok(ConnectionHandshakeStateAndProof {
        connection_state,
        encoded_connection_state_proof,
    })
}

struct ConnectionHandshakeStateAndProof {
    connection_state: ConnectionEnd,
    encoded_connection_state_proof: Bytes,
}
