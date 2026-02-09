#![doc = include_str!("../README.md")]

//! 1. Batch messages for the configured window for the client
//! 2. Attempt to fetch the proofs on the source chain for these messages at the latest finalized
//!    height of the source chain
//!   - For available proofs: fetch the update for the client on the intermediate chain to this
//!     height and submit it all (update + proof commitments) in one batch on the intermediate chain
//!   - For unavailable proofs: requeue and try 2. again
//! 3. Once the proofs are committed, update the client on the destination chain to a height >= the
//!    provable height of the commitments, build the messages to be sent to the counterparty with
//!    the proof of the proof of commitment, and submit it all in one batch on the destination chain

use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    convert,
    fmt::Debug,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use alloy::sol_types::SolValue;
use either::Either;
use futures::{StreamExt, TryFutureExt, TryStreamExt, stream::FuturesOrdered};
use ibc_union_spec::{
    Channel, ChannelState, ClientId, Connection, IbcUnion,
    datagram::*,
    event::{
        ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck, ConnectionOpenInit,
        ConnectionOpenTry,
    },
    path::*,
    query::PacketsByBatchHash,
};
use itertools::Itertools;
use jsonrpsee::{Extensions, core::async_trait};
use proof_lens_light_client_types::{ClientState, ConsensusState, Header};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::{debug, error, info, instrument, trace, warn};
use unionlabs::{
    ethereum::keccak256,
    ibc::core::client::height::Height,
    primitives::{Bytes, H256},
};
use voyager_sdk::{
    DefaultCmd, ExtensionsExt, VoyagerClient, anyhow,
    hook::simple_take_filter,
    into_value,
    message::{
        PluginMessage, VoyagerMessage,
        call::{
            FetchUpdateHeaders, SubmitTx, WaitForClientUpdate, WaitForHeight, WaitForHeightRelative,
        },
        data::{ChainEvent, Data, EventProvableHeight, IbcDatagram, OrderedHeaders},
    },
    plugin::Plugin,
    primitives::{ChainId, IbcSpec, IbcStorePathKey, QueryHeight},
    rpc::{PluginServer, RpcError, RpcErrorCode, RpcResult, types::PluginInfo},
    types::{ProofType, RawClientId},
    vm::{Op, call, conc, data, noop, pass::PassResult, promise, seq},
};

use crate::{
    call::{MakeProofCommitmentMsg, MakeTransactionBatchesWithUpdate, ModuleCall},
    callback::{MakeBatchTransaction, MakeProofLensClientUpdateWithMessages, ModuleCallback},
    data::{
        BatchableEvent, CommittableEvent, EventBatch, ModuleData, MsgWithCommitmentMsg,
        UnsaturatedMsgWithStoreKey,
    },
};

pub mod call;
pub mod callback;
pub mod data;

#[derive(Debug, Clone)]
pub struct Module {
    /// The destination chain (i.e. where the messages will be sent to)
    pub chain_id: ChainId,
    pub client_configs: ClientConfigs,
}

#[derive(Debug, Clone)]
pub enum ClientConfigs {
    Any(ClientConfig),
    Many(HashMap<ClientId, ClientConfig>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub client_configs: ClientConfigsSerde,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientConfig {
    pub min_batch_size: usize,
    pub max_batch_size: usize,
    pub max_wait_time: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ClientConfigsSerde {
    Any(ClientConfig),
    Many(Vec<SpecificClientConfig>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecificClientConfig {
    pub client_id: ClientId,
    pub min_batch_size: usize,
    pub max_batch_size: usize,
    pub max_wait_time: Duration,
}

impl SpecificClientConfig {
    fn into_config(self) -> (ClientId, ClientConfig) {
        (
            self.client_id,
            ClientConfig {
                min_batch_size: self.min_batch_size,
                max_batch_size: self.max_batch_size,
                max_wait_time: self.max_wait_time,
            },
        )
    }
}

pub trait IbcSpecExt: IbcSpec + Clone {
    type BatchableEvent: TryFrom<Self::Event, Error = ()>
        + Eq
        + Serialize
        + DeserializeOwned
        + Debug
        + Clone;

    fn proof_height(msg: &Self::Datagram) -> u64;

    fn event_name(msg: &Self::BatchableEvent) -> &'static str;
}

impl IbcSpecExt for IbcUnion {
    type BatchableEvent = crate::data::CommittableEvent;

    fn proof_height(msg: &Self::Datagram) -> u64 {
        msg.proof_height()
            .expect("all batchable messages have a proof")
            .height()
    }

    fn event_name(msg: &Self::BatchableEvent) -> &'static str {
        match msg {
            CommittableEvent::ConnectionOpenInit(_) => "connection_open_init",
            CommittableEvent::ConnectionOpenTry(_) => "connection_open_try",
            CommittableEvent::ConnectionOpenAck(_) => "connection_open_ack",
            CommittableEvent::ChannelOpenInit(_) => "channel_open_init",
            CommittableEvent::ChannelOpenTry(_) => "channel_open_try",
            CommittableEvent::ChannelOpenAck(_) => "channel_open_ack",
            CommittableEvent::PacketSend(_) => "packet_send",
            CommittableEvent::BatchSend(_) => "batch_send",
            CommittableEvent::WriteAck(_) => "write_ack",
        }
    }
}

impl ClientConfigs {
    fn new(configs: ClientConfigsSerde) -> Self {
        match configs {
            ClientConfigsSerde::Any(client_config) => ClientConfigs::Any(client_config),
            ClientConfigsSerde::Many(vec) => {
                ClientConfigs::Many(vec.into_iter().map(|s| s.into_config()).collect())
            }
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
                let clients_json =
                    serde_json::to_string(&many.keys().map(|k| (k, ())).collect::<HashMap<_, _>>())
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

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        Ok(Module::new(config))
    }

    fn info(config: Self::Config) -> PluginInfo {
        let module = Module::new(config);

        PluginInfo {
            name: module.plugin_name(),
            interest_filter: simple_take_filter(format!(
                r#"
if ."@type" == "data" then
    ."@value" as $data |

    # pull all ibc events that cause an action on this chain (i.e. where we are the destination)
    # the counterparty of the event origin is the destination

    ($data."@type" == "plugin"
        and $data."@value".plugin == "{plugin_name}"
        and $data."@value".message."@type" == "batch_events_union"
    ) or
    if $data."@type" == "ibc_event" and $data."@value".counterparty_chain_id == "{chain_id}" and $data."@value".ibc_spec_id == "{ibc_union_id}" then
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
            and ($event_data.connection.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_try"
            and ($event_data.connection.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "channel_open_ack"
            and ($event_data.connection.counterparty_client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "packet_send"
            and ($event_data.packet.destination_channel.connection.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "batch_send"
            and ($event_data.destination_channel.connection.client_id as $client_id | {clients_filter})
        ) or (
            $event_type == "write_ack"
            and ($event_data.packet.source_channel.connection.client_id as $client_id | {clients_filter})
        )
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
                ibc_union_id = IbcUnion::ID,
            )),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

pub fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
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
        match msg {
            ModuleCall::MakeTransactionBatchesWithUpdate(mk) => {
                self.make_transaction_batches_with_update(e.voyager_client()?, mk)
                    .await
            }
            ModuleCall::MakeProofCommitmentMsg(mk) => {
                self.make_proof_commitment_message(e.voyager_client()?, mk)
                    .await
            }
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
            ModuleCallback::MakeIbcMessagesFromUpdate(_) => {
                // self.make_ibc_message_from_update(e.voyager_client()?, cb, datas)
                //     .await
                todo!()
            }
            ModuleCallback::MakeBatchTransaction(cb) => {
                self.make_batch_transaction(e.voyager_client()?, cb, datas)
                    .await
            }
            ModuleCallback::MakeProofLensClientUpdateWithMessages(cb) => {
                self.make_proof_lens_client_update_with_messages(e.voyager_client()?, cb, datas)
                    .await
            }
        }
    }
}

impl Module {
    async fn run_pass_internal(
        &self,
        e: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        let mut batchers = HashMap::<ClientId, Vec<(usize, BatchableEvent)>>::new();

        for (idx, msg) in msgs.into_iter().enumerate() {
            let Op::Data(msg) = msg else {
                error!("unexpected message: {msg:?}");
                continue;
            };

            match ChainEvent::try_from(msg) {
                Ok(chain_event) => {
                    let first_seen_at: u64 = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                        .try_into()
                        .expect("how many milliseconds can there be man");

                    if let Some(full_ibc_event) =
                        chain_event.decode_event::<IbcUnion>().transpose().unwrap()
                    {
                        // client_id is the client id of the client on this chain (we are the
                        // counterparty from the perspective of the chain where the event was
                        // emitted); this is the client that will need to be updated before
                        // the resultant message of this event can be sent to the destination chain

                        let client_id = full_ibc_event
                            .counterparty_client_id()
                            .expect("all batchable messages have a counterparty");

                        trace!(%client_id, "batching event");

                        batchers.entry(client_id).or_default().push((
                            idx,
                            BatchableEvent {
                                first_seen_at,
                                provable_height: chain_event.provable_height,
                                // TODO: Handle this more gracefully
                                event: full_ibc_event.try_into().unwrap(),
                            },
                        ));
                    }
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

                        Ok(msg) => {
                            error!("unexpected message: {msg:?}");
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
            .flat_map(|(client_id, events)| self.split_ready(client_id, events))
            .partition_map::<Vec<_>, Vec<_>, _, _, _>(convert::identity);

        trace!(
            ready = ready.len(),
            optimize_further = optimize_further.len()
        );

        let v = e.voyager_client()?;

        let (errored, ready) = ready
            .into_iter()
            .into_group_map()
            .into_iter()
            .map(|(client_id, events)| self.mk_ready_ops(v, client_id, events))
            .collect::<FuturesOrdered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .partition_map::<Vec<_>, Vec<_>, _, _, _>(Either::from);

        trace!(ready = ready.len(), errored = errored.len(),);

        Ok(PassResult {
            optimize_further: optimize_further
                .into_iter()
                .chain(errored.into_iter().flatten())
                .collect(),
            ready,
        })
    }
}

/// Calls
impl Module {
    #[instrument(
        skip_all,
        fields(
            %origin_chain_id,
            %origin_chain_proof_height,
            target_chain_id = %self.chain_id,
            msg = IbcUnion::event_name(&event)
        )
    )]
    pub async fn make_proof_commitment_message(
        &self,
        v: &VoyagerClient,
        MakeProofCommitmentMsg {
            origin_chain_id,
            origin_chain_proof_height,
            event,
        }: MakeProofCommitmentMsg,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match event {
            CommittableEvent::ConnectionOpenInit(ConnectionOpenInit {
                connection_id,
                client_id,
                counterparty_client_id,
            }) => {
                let proof_lens_client_state = self
                    .query_proof_lens_client_state(v, counterparty_client_id)
                    .await?;

                let path = ConnectionPath { connection_id };

                // the connection end as stored by the origin chain after open_init
                let connection = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(connection = %into_value(&connection), "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commit(connection.abi_encode_params()).into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::ConnectionOpenTry(
                            MsgConnectionOpenTry {
                                client_id: counterparty_client_id,
                                proof_height: origin_chain_proof_height.height(),
                                counterparty_client_id: client_id,
                                counterparty_connection_id: connection_id,
                                proof_init: b"".into(),
                            },
                            path,
                        ),
                    }),
                )))
            }

            CommittableEvent::ConnectionOpenTry(ConnectionOpenTry {
                connection_id,
                counterparty_client_id,
                counterparty_connection_id,
                ..
            }) => {
                let proof_lens_client_state = self
                    .query_proof_lens_client_state(v, counterparty_client_id)
                    .await?;

                let path = ConnectionPath { connection_id };

                // the connection end as stored by the origin chain after open_try
                let connection = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(connection = %into_value(&connection), "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commit(connection.abi_encode_params()).into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::ConnectionOpenAck(
                            MsgConnectionOpenAck {
                                connection_id: counterparty_connection_id,
                                counterparty_connection_id: connection_id,
                                proof_height: origin_chain_proof_height.height(),
                                proof_try: b"".into(),
                            },
                            path,
                        ),
                    }),
                )))
            }

            CommittableEvent::ConnectionOpenAck(ConnectionOpenAck {
                connection_id,
                counterparty_client_id,
                counterparty_connection_id,
                ..
            }) => {
                let proof_lens_client_state = self
                    .query_proof_lens_client_state(v, counterparty_client_id)
                    .await?;

                let path = ConnectionPath { connection_id };

                // the connection end as stored by the origin chain after open_ack
                let connection = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(connection = %into_value(&connection), "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commit(connection.abi_encode_params()).into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::ConnectionOpenConfirm(
                            MsgConnectionOpenConfirm {
                                connection_id: counterparty_connection_id,
                                proof_height: origin_chain_proof_height.height(),
                                proof_ack: b"".into(),
                            },
                            path,
                        ),
                    }),
                )))
            }

            CommittableEvent::ChannelOpenInit(ChannelOpenInit {
                port_id,
                channel_id,
                connection:
                    Connection {
                        counterparty_client_id,
                        counterparty_connection_id,
                        ..
                    },
                version,
                counterparty_port_id,
            }) => {
                let proof_lens_client_state = self
                    .query_proof_lens_client_state(v, counterparty_client_id)
                    .await?;

                let path = ChannelPath { channel_id };

                let channel = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(channel = %into_value(&channel), "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commit(channel.abi_encode()).into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::ChannelOpenTry(
                            MsgChannelOpenTry {
                                port_id: counterparty_port_id,
                                channel: Channel {
                                    state: ChannelState::TryOpen,
                                    counterparty_channel_id: Some(channel_id),
                                    counterparty_port_id: port_id,
                                    connection_id: counterparty_connection_id.unwrap(),
                                    version: version.clone(),
                                },
                                counterparty_version: version,
                                proof_height: origin_chain_proof_height.height(),
                                proof_init: b"".into(),
                            },
                            path,
                        ),
                    }),
                )))
            }

            CommittableEvent::ChannelOpenTry(ChannelOpenTry {
                channel_id,
                counterparty_channel_id,
                connection:
                    Connection {
                        counterparty_client_id,
                        ..
                    },
                version,
                ..
            }) => {
                let proof_lens_client_state = self
                    .query_proof_lens_client_state(v, counterparty_client_id)
                    .await?;

                let path = ChannelPath { channel_id };

                let channel = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(channel = %into_value(&channel), "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commit(channel.abi_encode()).into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::ChannelOpenAck(
                            MsgChannelOpenAck {
                                channel_id: counterparty_channel_id,
                                counterparty_channel_id: channel_id,
                                counterparty_version: version,
                                proof_height: origin_chain_proof_height.height(),
                                proof_try: b"".into(),
                            },
                            path,
                        ),
                    }),
                )))
            }

            CommittableEvent::ChannelOpenAck(ChannelOpenAck {
                channel_id,
                counterparty_channel_id,
                connection:
                    Connection {
                        counterparty_client_id,
                        ..
                    },
                ..
            }) => {
                let proof_lens_client_state = self
                    .query_proof_lens_client_state(v, counterparty_client_id)
                    .await?;

                let path = ChannelPath { channel_id };

                let channel = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(channel = %into_value(&channel), "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commit(channel.abi_encode()).into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::ChannelOpenConfirm(
                            MsgChannelOpenConfirm {
                                channel_id: counterparty_channel_id,
                                proof_height: origin_chain_proof_height.height(),
                                proof_ack: b"".into(),
                            },
                            path,
                        ),
                    }),
                )))
            }

            CommittableEvent::PacketSend(event) => {
                let packet = event.packet();

                let proof_lens_client_state = self
                    .query_proof_lens_client_state(
                        v,
                        event.packet.destination_channel.connection.client_id,
                    )
                    .await?;

                let path = BatchPacketsPath::from_packet(&packet);

                let commitment = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(%commitment, "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commitment.into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::PacketRecv(
                            MsgPacketRecv {
                                packets: vec![packet],
                                relayer_msgs: vec![vec![].into()],
                                proof_height: origin_chain_proof_height.height(),
                                proof: b"".into(),
                            },
                            path,
                        ),
                    }),
                )))
            }

            CommittableEvent::BatchSend(event) => {
                let proof_lens_client_state = self
                    .query_proof_lens_client_state(
                        v,
                        event.destination_channel.connection.client_id,
                    )
                    .await?;

                let mut packets = v
                    .query(
                        origin_chain_id.clone(),
                        PacketsByBatchHash {
                            channel_id: event.source_channel.channel_id,
                            batch_hash: event.batch_hash,
                        },
                    )
                    .await?
                    .packets;

                packets.sort_by_cached_key(|packet| packet.hash());

                let path = BatchPacketsPath::from_packets(&packets);

                let commitment = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(%commitment, "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commitment.into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::PacketRecv(
                            MsgPacketRecv {
                                relayer_msgs: vec![vec![].into(); packets.len()],
                                packets,
                                proof: b"".into(),
                                proof_height: origin_chain_proof_height.height(),
                            },
                            path,
                        ),
                    }),
                )))
            }

            CommittableEvent::WriteAck(event) => {
                let packet = event.packet();

                let proof_lens_client_state = self
                    .query_proof_lens_client_state(
                        v,
                        event.packet.source_channel.connection.client_id,
                    )
                    .await?;

                let path = BatchReceiptsPath::from_packet(&packet);

                let commitment = v
                    .query_ibc_state(origin_chain_id.clone(), QueryHeight::Latest, path.clone())
                    .await?;

                debug!(%commitment, "committed value");

                let encoded_proof = self
                    .query_and_encode_proof(
                        v,
                        &origin_chain_id,
                        &proof_lens_client_state,
                        origin_chain_proof_height,
                        &path,
                    )
                    .await?;

                Ok(data(PluginMessage::new(
                    self.plugin_name(),
                    ModuleData::from(MsgWithCommitmentMsg {
                        commitment_msg: MsgCommitMembershipProof {
                            client_id: proof_lens_client_state.l2_client_id,
                            proof_height: origin_chain_proof_height.height(),
                            proof: encoded_proof,
                            path: path.key().into(),
                            value: commitment.into(),
                        },
                        msg: UnsaturatedMsgWithStoreKey::PacketAcknowledgement(
                            MsgPacketAcknowledgement {
                                packets: vec![packet],
                                acknowledgements: vec![event.acknowledgement],
                                proof: b"".into(),
                                proof_height: origin_chain_proof_height.height(),
                            },
                            path,
                        ),
                    }),
                )))
            }
        }
    }

    #[instrument(skip_all, fields(client_id, batches = batches.len()))]
    pub async fn make_transaction_batches_with_update(
        &self,
        v: &VoyagerClient,
        MakeTransactionBatchesWithUpdate { client_id, batches }: MakeTransactionBatchesWithUpdate,
    ) -> RpcResult<Op<VoyagerMessage>> {
        #[derive(Debug)]
        enum TargetHeights {
            None,
            Min(Height),
            Exact(Vec<Height>),
        }

        let client_state_meta = v
            .client_state_meta::<IbcUnion>(self.chain_id.clone(), QueryHeight::Latest, client_id)
            .await?;

        let latest_height = v
            .query_latest_height(client_state_meta.counterparty_chain_id.clone(), true)
            .await?;

        let target_heights = batches
            .iter()
            .flatten()
            .map(|e| e.provable_height)
            .try_fold(TargetHeights::None, |acc, elem| match (elem, acc) {
                (EventProvableHeight::Min(elem), TargetHeights::None) => {
                    Ok(TargetHeights::Min(elem))
                }
                (EventProvableHeight::Exactly(elem), TargetHeights::None) => {
                    Ok(TargetHeights::Exact(vec![elem]))
                }

                (EventProvableHeight::Min(elem), TargetHeights::Min(acc)) => {
                    Ok(TargetHeights::Min(elem.max(acc)))
                }

                (EventProvableHeight::Exactly(elem), TargetHeights::Exact(acc)) => Ok(
                    TargetHeights::Exact(acc.into_iter().chain([elem]).collect()),
                ),

                (elem, acc) => Err(RpcError::fatal_from_message(format!(
                    "cannot mix exact and min update heights in a \
                    single instance of this plugin: {elem:?}, {acc:?}"
                ))),
            })?;

        debug!(?target_heights);

        let mut ops = vec![];

        // note that proofs must be fetched BEFORE updating the client; we have a small window where proofs are available
        // this means that this plugin is only compatible with clients that allow updating to an exact height (i.e. we can't do a proof lens client of ethereum)
        match target_heights {
            TargetHeights::Min(target_height) => {
                info!(
                    "min height of all events is {target_height}, latest height is {latest_height}"
                );

                ops.push(
                    self.fetch_commitment_proofs(v, client_id, batches, latest_height)
                        .await?,
                );
            }
            TargetHeights::None => todo!(),
            TargetHeights::Exact(_) => todo!(),
        }

        Ok(conc(ops))
    }

    #[instrument(skip_all, fields(client_id, unsaturated_msgs = unsaturated_msgs.len()))]
    async fn make_proof_lens_client_update_with_messages(
        &self,
        v: &VoyagerClient,
        MakeProofLensClientUpdateWithMessages {
            client_id,
            unsaturated_msgs,
        }: MakeProofLensClientUpdateWithMessages,
        datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let update: Option<OrderedHeaders> = datas
            .into_iter()
            .exactly_one()
            .map_err(|found| serde_json::to_string(&found.collect::<Vec<_>>()).unwrap())
            .and_then(|d| {
                d.try_into()
                    .map_err(|found| serde_json::to_string(&found).unwrap())
            })
            .ok();

        let proof_lens_client_info = v
            .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
            .await?;

        let proof_lens_client_state = v
            .decode_client_state::<IbcUnion, ClientState>(
                proof_lens_client_info.client_type.clone(),
                proof_lens_client_info.ibc_interface.clone(),
                v.query_ibc_state(
                    self.chain_id.clone(),
                    QueryHeight::Latest,
                    ClientStatePath { client_id },
                )
                .await?,
            )
            .await?;

        debug!(
            l2_chain_id = %proof_lens_client_state.l2_chain_id,
            l1_client_id = %proof_lens_client_state.l1_client_id,
            l2_client_id = %proof_lens_client_state.l2_client_id,
            l2_latest_height = %proof_lens_client_state.l2_latest_height,
            timestamp_offset = %proof_lens_client_state.timestamp_offset,
            "proof lens client state",
        );

        let l1_client_state_meta = v
            .client_state_meta::<IbcUnion>(
                self.chain_id.clone(),
                QueryHeight::Latest,
                proof_lens_client_state.l1_client_id,
            )
            .await?;

        let l1_client_info = v
            .client_info::<IbcUnion>(self.chain_id.clone(), proof_lens_client_state.l1_client_id)
            .await?;

        let l1_chain_id = l1_client_state_meta.counterparty_chain_id;

        let commitment_height = unsaturated_msgs[0].proof_height();

        info!(commitment_height);

        let consensus_state_path = ConsensusStatePath {
            client_id,
            height: commitment_height,
        };

        let raw_consensus_state = v
            .query_ibc_state(
                self.chain_id.clone(),
                QueryHeight::Latest,
                consensus_state_path,
            )
            .await?;

        debug!(%raw_consensus_state);

        let proof_lens_consensus_state = v
            .decode_consensus_state::<IbcUnion, ConsensusState>(
                proof_lens_client_info.client_type.clone(),
                proof_lens_client_info.ibc_interface.clone(),
                raw_consensus_state,
            )
            .await?;

        debug!(
            l1_height = proof_lens_consensus_state.l1_height,
            timestamp = %proof_lens_consensus_state.timestamp,
            raw_l2_consensus_state = %proof_lens_consensus_state.raw_l2_consensus_state,
            "proof lens consensus state",
        );

        let l1_proof_height = match update {
            Some(ref update) => {
                serde_json::from_value::<Header>(
                    update.headers.last().expect("not empty").1.clone(),
                )
                .expect("valid header")
                .l1_height
            }
            None => {
                if proof_lens_consensus_state.l1_height == 0 {
                    return Err(RpcError::fatal_from_message(format!(
                        "no update provided in callback data and no consensus state \
                        found at height {commitment_height} for client {client_id} on {}",
                        self.chain_id,
                    )));
                }

                proof_lens_consensus_state.l1_height
            }
        };

        debug!(l1_proof_height);

        let mut saturated_msgs = Vec::with_capacity(unsaturated_msgs.len());

        for unsaturated_msg in unsaturated_msgs {
            let committed_path = unsaturated_msg.path();

            let commitment_path = MembershipProofPath {
                client_id: proof_lens_client_state.l2_client_id,
                proof_height: commitment_height,
                path: committed_path.key().into(),
            };

            debug!(
                committed_path = %into_value(&committed_path),
                committed_key = %committed_path.key(),
                commitment_path = %into_value(&commitment_path),
                commitment_key = %commitment_path.key(),
            );

            let raw_proof = v
                .query_ibc_proof(
                    l1_chain_id.clone(),
                    QueryHeight::Specific(Height::new(l1_proof_height)),
                    commitment_path,
                )
                .await?
                .into_result()?;

            if raw_proof.proof_type != ProofType::Membership {
                return Err(RpcError::retryable_from_message(
                    "proof is not a membership proof",
                ));
            }

            let encoded_proof = v
                .encode_proof::<IbcUnion>(
                    l1_client_info.client_type.clone(),
                    l1_client_info.ibc_interface.clone(),
                    raw_proof.proof,
                )
                .await?;

            let msg: Datagram = match unsaturated_msg {
                UnsaturatedMsgWithStoreKey::ConnectionOpenTry(mut msg, _) => {
                    msg.proof_init = encoded_proof;
                    msg.into()
                }
                UnsaturatedMsgWithStoreKey::ConnectionOpenAck(mut msg, _) => {
                    msg.proof_try = encoded_proof;
                    msg.into()
                }
                UnsaturatedMsgWithStoreKey::ConnectionOpenConfirm(mut msg, _) => {
                    msg.proof_ack = encoded_proof;
                    msg.into()
                }
                UnsaturatedMsgWithStoreKey::ChannelOpenTry(mut msg, _) => {
                    msg.proof_init = encoded_proof;
                    msg.into()
                }
                UnsaturatedMsgWithStoreKey::ChannelOpenAck(mut msg, _) => {
                    msg.proof_try = encoded_proof;
                    msg.into()
                }
                UnsaturatedMsgWithStoreKey::ChannelOpenConfirm(mut msg, _) => {
                    msg.proof_ack = encoded_proof;
                    msg.into()
                }
                UnsaturatedMsgWithStoreKey::PacketRecv(mut msg, _) => {
                    msg.proof = encoded_proof;
                    msg.into()
                }
                UnsaturatedMsgWithStoreKey::PacketAcknowledgement(mut msg, _) => {
                    msg.proof = encoded_proof;
                    msg.into()
                }
            };

            saturated_msgs.push(msg);
        }

        Ok(call(SubmitTx {
            chain_id: self.chain_id.clone(),
            datagrams: update
                .into_iter()
                .flat_map(|h| h.headers)
                .map(|(_, header)| {
                    v.encode_header::<IbcUnion>(
                        proof_lens_client_info.client_type.clone(),
                        proof_lens_client_info.ibc_interface.clone(),
                        header,
                    )
                    .map_ok(|encoded_header| {
                        IbcUnion::update_client_datagram(client_id, encoded_header)
                    })
                })
                .collect::<FuturesOrdered<_>>()
                .try_collect::<Vec<_>>()
                .await?
                .into_iter()
                .chain(saturated_msgs)
                .map(IbcDatagram::new::<IbcUnion>)
                .collect::<Vec<_>>(),
        }))
    }
}

/// Callbacks
impl Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id, client_id, datas_len = datas.len()))]
    pub async fn make_batch_transaction(
        &self,
        v: &VoyagerClient,
        MakeBatchTransaction { client_id }: MakeBatchTransaction,
        datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        if datas.is_empty() {
            warn!(
                "no IBC messages in queue! this likely means that all of the IBC messages that were queued to be sent were already sent to the destination chain"
            );

            return Ok(noop());
        }

        let mut msgs = vec![];
        let mut events_no_proof_available = vec![];
        let mut update = None;

        for data in datas {
            match data {
                Data::OrderedHeaders(d) => {
                    if update.replace(d).is_some() {
                        return Err(RpcError::fatal_from_message(
                            "multiple updates found in data",
                        ));
                    }
                }
                Data::Plugin(d) => match d
                    .downcast::<ModuleData>(self.plugin_name())
                    .map_err(|_| RpcError::fatal_from_message("invalid plugin data"))?
                {
                    ModuleData::BatchEvents(_) => {
                        return Err(RpcError::fatal_from_message("unexpected plugin data"));
                    }
                    ModuleData::ProofUnavailable(d) => events_no_proof_available.push(d),
                    ModuleData::MsgWithCommitmentMsg(d) => msgs.push(d),
                },
                _ => return Err(RpcError::fatal_from_message("unexpected data")),
            }
        }

        let (commitment_msgs, ibc_msgs) = msgs
            .into_iter()
            .map(|msg| (msg.commitment_msg, msg.msg))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let mut commitment_msgs = commitment_msgs.into_iter().peekable();

        let proof_lens_client_info = v
            .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
            .await?;

        let proof_lens_client_state = v
            .decode_client_state::<IbcUnion, ClientState>(
                proof_lens_client_info.client_type.clone(),
                proof_lens_client_info.ibc_interface.clone(),
                v.query_ibc_state(
                    self.chain_id.clone(),
                    QueryHeight::Latest,
                    ClientStatePath { client_id },
                )
                .await?,
            )
            .await?;

        debug!(
            l2_chain_id = %proof_lens_client_state.l2_chain_id,
            l1_client_id = %proof_lens_client_state.l1_client_id,
            l2_client_id = %proof_lens_client_state.l2_client_id,
            l2_latest_height = %proof_lens_client_state.l2_latest_height,
            timestamp_offset = %proof_lens_client_state.timestamp_offset,
            "proof lens client state",
        );

        let l2_client_info = v
            .client_info::<IbcUnion>(
                v.client_state_meta::<IbcUnion>(
                    self.chain_id.clone(),
                    QueryHeight::Latest,
                    proof_lens_client_state.l1_client_id,
                )
                .await?
                .counterparty_chain_id,
                proof_lens_client_state.l2_client_id,
            )
            .await?;

        let events_no_proof_available_msg = if !events_no_proof_available.is_empty() {
            info!(
                count = events_no_proof_available.len(),
                "found events with no proof available"
            );

            todo!();

            // let latest_height = voyager_client
            //     .query_latest_height(self.chain_id.clone(), true)
            //     .await?;

            // let client_state_meta = voyager_client
            //     .client_state_meta::<IbcUnion>(
            //         self.chain_id.clone(),
            //         QueryHeight::Latest,
            //         client_id,
            //     )
            //     .await?;

            // info!(
            //     count = events_no_proof_available.len(),
            //     "updating client {} to {latest_height}", client_id,
            // );

            // Some(promise(
            //     [call(FetchUpdateHeaders {
            //         client_type: client_info.client_type.clone(),
            //         counterparty_chain_id: self.chain_id.clone(),
            //         chain_id: self.chain_id.clone(),
            //         client_id: RawClientId::new(client_id),
            //         update_from: client_state_meta.counterparty_height,
            //         update_to: latest_height,
            //     })],
            //     [],
            //     PluginMessage::new(
            //         self.plugin_name(),
            //         ModuleCallback::from(MakeIbcMessagesFromUpdate {
            //             client_id,
            //             batches: vec![
            //                 events_no_proof_available
            //                     .into_iter()
            //                     .map(|e| e.event)
            //                     .collect(),
            //             ],
            //         }),
            //     ),
            // ))
        } else {
            None
        };

        let l1_chain_id = {
            let l1_client_state_meta = v
                .client_state_meta::<IbcUnion>(
                    self.chain_id.clone(),
                    QueryHeight::Latest,
                    proof_lens_client_state.l1_client_id,
                )
                .await?;

            l1_client_state_meta.counterparty_chain_id
        };

        let continuation = match commitment_msgs.peek() {
            Some(msg) => promise(
                [call(FetchUpdateHeaders {
                    client_type: proof_lens_client_info.client_type,
                    chain_id: ChainId::new(proof_lens_client_state.l2_chain_id.clone()),
                    counterparty_chain_id: self.chain_id.clone(),
                    client_id: RawClientId::new(client_id),
                    update_from: Height::new(proof_lens_client_state.l2_latest_height),
                    update_to: Height::new(msg.proof_height),
                })],
                [],
                PluginMessage::new(
                    self.plugin_name(),
                    ModuleCallback::from(MakeProofLensClientUpdateWithMessages {
                        client_id,
                        unsaturated_msgs: ibc_msgs,
                    }),
                ),
            ),
            None => noop(),
        };

        let msg = match update {
            Some(update) => call(SubmitTx {
                chain_id: l1_chain_id.clone(),
                datagrams: update
                    .headers
                    .into_iter()
                    .map(|(_, header)| {
                        v.encode_header::<IbcUnion>(
                            l2_client_info.client_type.clone(),
                            l2_client_info.ibc_interface.clone(),
                            header,
                        )
                        .map_ok(|encoded_header| {
                            IbcUnion::update_client_datagram(
                                proof_lens_client_state.l2_client_id,
                                encoded_header,
                            )
                        })
                    })
                    .collect::<FuturesOrdered<_>>()
                    .try_collect::<Vec<_>>()
                    .await?
                    .into_iter()
                    .chain(commitment_msgs.map(Datagram::from))
                    .map(IbcDatagram::new::<IbcUnion>)
                    .collect::<Vec<_>>(),
            }),
            None => {
                if commitment_msgs.len() == 0 {
                    noop()
                } else {
                    // TODO: We can probably relax this in the future if we want to reuse this
                    // module to work with all IBC messages NOTE: We assume that
                    // all of the IBC messages were generated against the same consensus height
                    let required_consensus_height = commitment_msgs
                        .peek()
                        .expect("msgs is non-empty; qed;")
                        .proof_height;

                    seq([
                        call(WaitForClientUpdate::new::<IbcUnion>(
                            &self.chain_id,
                            &client_id,
                            Height::new(required_consensus_height),
                        )),
                        call(SubmitTx {
                            chain_id: l1_chain_id.clone(),
                            datagrams: commitment_msgs.map(IbcDatagram::new::<IbcUnion>).collect(),
                        }),
                    ])
                }
            }
        };

        Ok(conc(events_no_proof_available_msg.into_iter().chain([
            seq([
                msg,
                call(WaitForHeightRelative {
                    chain_id: l1_chain_id,
                    height_diff: 1,
                    finalized: false,
                }),
                continuation,
            ]),
        ])))
    }
}

/// Utils
impl Module {
    #[instrument(
        skip_all,
        fields(
            chain_id = %self.chain_id,
            %client_id,
            batches = batches.len(),
            %proof_height,
        )
    )]
    pub async fn fetch_commitment_proofs(
        &self,
        v: &VoyagerClient,
        client_id: ClientId,
        batches: Vec<Vec<BatchableEvent>>,
        proof_height: Height,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let proof_lens_client_info = v
            .client_info::<IbcUnion>(self.chain_id.clone(), client_id)
            .await?;

        let proof_lens_client_state = v
            .decode_client_state::<IbcUnion, ClientState>(
                proof_lens_client_info.client_type.clone(),
                proof_lens_client_info.ibc_interface.clone(),
                v.query_ibc_state(
                    self.chain_id.clone(),
                    QueryHeight::Latest,
                    ClientStatePath { client_id },
                )
                .await?,
            )
            .await?;

        debug!(
            l2_chain_id = %proof_lens_client_state.l2_chain_id,
            l1_client_id = %proof_lens_client_state.l1_client_id,
            l2_client_id = %proof_lens_client_state.l2_client_id,
            l2_latest_height = %proof_lens_client_state.l2_latest_height,
            timestamp_offset = %proof_lens_client_state.timestamp_offset,
            "proof lens client state",
        );

        let l1_client_state_meta = v
            .client_state_meta::<IbcUnion>(
                self.chain_id.clone(),
                QueryHeight::Latest,
                proof_lens_client_state.l1_client_id,
            )
            .await?;

        let l1_chain_id = l1_client_state_meta.counterparty_chain_id;

        let l2_client_info = v
            .client_info::<IbcUnion>(l1_chain_id.clone(), proof_lens_client_state.l2_client_id)
            .await?;

        let l2_client_state_meta = v
            .client_state_meta::<IbcUnion>(
                l1_chain_id.clone(),
                QueryHeight::Latest,
                proof_lens_client_state.l2_client_id,
            )
            .await?;

        let (update_to, update) = if l2_client_state_meta.counterparty_height >= proof_height {
            info!(
                "l2 client {client_id} on {l1_chain_id} has already been updated to a height \
                >= the desired target height ({} >= {proof_height})",
                l2_client_state_meta.counterparty_height,
            );

            (l2_client_state_meta.counterparty_height, None)
        } else {
            info!("l2 client {client_id} on {l1_chain_id} will be updated to {proof_height}");

            (
                proof_height,
                Some(call(FetchUpdateHeaders {
                    client_type: l2_client_info.client_type.clone(),
                    counterparty_chain_id: l1_chain_id,
                    chain_id: ChainId::new(proof_lens_client_state.l2_chain_id.clone()),
                    client_id: RawClientId::new(proof_lens_client_state.l2_client_id),
                    update_from: l2_client_state_meta.counterparty_height,
                    update_to: proof_height,
                })),
            )
        };

        Ok(conc(batches.into_iter().enumerate().map(|(idx, batch)| {
            promise(
                [conc(
                    batch
                        .into_iter()
                        .map(|batchable_event| {
                            let origin_chain_id =
                                ChainId::new(proof_lens_client_state.l2_chain_id.clone());

                            debug!(
                                %origin_chain_id,
                                target_chain_id = %self.chain_id,
                                event = IbcUnion::event_name(&batchable_event.event),
                                provable_height = ?batchable_event.provable_height,
                                first_seen_at = batchable_event.first_seen_at,
                                "batching event"
                            );

                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeProofCommitmentMsg {
                                    origin_chain_id,
                                    origin_chain_proof_height: update_to,
                                    event: batchable_event.event,
                                }),
                            ))
                        })
                        .chain((idx == 0).then(|| update.clone()).flatten()),
                )],
                [],
                PluginMessage::new(
                    self.plugin_name(),
                    ModuleCallback::from(MakeBatchTransaction { client_id }),
                ),
            )
        })))
    }

    #[allow(clippy::type_complexity)] // skill issue
    fn split_ready(
        &self,
        client_id: ClientId,
        mut events: Vec<(usize, BatchableEvent)>,
    ) -> Vec<
        Either<
            // ready
            (ClientId, (Vec<usize>, Vec<BatchableEvent>)),
            // optimize further
            (Vec<usize>, Op<VoyagerMessage>, String),
        >,
    > {
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

        events.sort_by_key(|e| *e.1.provable_height.height());
        overdue_events.sort_by_key(|e| *e.1.provable_height.height());

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
                    // this batch is ready to send out, we need to fetch an update for the client on
                    // our chain and turn the events into `IbcMessage`s.
                    //
                    // in order to do this, we first need to figure out what height the client is
                    // at, and request an update from that height to a height >= the highest height
                    // of all of the messages in this batch. note that we can't
                    // request a *specific* height to update to, since not all chains provide this
                    // functionality (ethereum being a notable one) - we instead need to wait for
                    // the update to be constructed, and then use the new trusted height of the
                    // update to fetch our proofs from.
                    //
                    // this will be done in a multi-step aggregation, where first we fetch the
                    // update, then commit the proofs on the intermediate L1 chain, then construct
                    // the messages, and then turn that into a batch transaction.
                    Either::Left((client_id, (idxs, events)))
                } else {
                    Either::Right((
                        idxs,
                        data(PluginMessage::new(
                            self.plugin_name(),
                            ModuleData::from(EventBatch { client_id, events }),
                        )),
                        self.plugin_name(),
                    ))
                }
            })
            .collect::<Vec<_>>()
    }

    #[expect(unstable_name_collisions)] // for Itertools::intersperse
    #[expect(clippy::type_complexity)] // it's not that bad bro
    #[instrument(skip_all, fields(client_id, events = events.len()))]
    async fn mk_ready_ops(
        &self,
        v: &VoyagerClient,
        client_id: ClientId,
        events: Vec<(Vec<usize>, Vec<BatchableEvent>)>,
    ) -> Result<(Vec<usize>, Op<VoyagerMessage>), Vec<(Vec<usize>, Op<VoyagerMessage>, String)>>
    {
        // the height on the counterparty chain that all of the events in these batches are provable
        // at we only want to generate one update for all of these batches
        let (min_target_height, exact_target_heights) = events
            .iter()
            .flat_map(|x| &x.1)
            .map(|e| e.provable_height)
            .fold((None, BTreeSet::new()), |mut acc, elem| match elem {
                EventProvableHeight::Min(height) => (
                    Some(acc.0.map(|h: Height| h.min(height)).unwrap_or(height)),
                    acc.1,
                ),
                EventProvableHeight::Exactly(height) => {
                    acc.1.insert(height);
                    acc
                }
            });

        info!(
            "target height of updates for batch is min {}, exact [{}]",
            min_target_height.map_or("<none>".to_string(), |h: Height| h.to_string()),
            exact_target_heights
                .iter()
                .map(|h| h.to_string())
                .intersperse(",".to_string())
                .collect::<String>(),
        );

        debug!(%client_id, "querying client state meta for client");

        let client_state_meta = match v
            .client_state_meta::<IbcUnion>(self.chain_id.clone(), QueryHeight::Latest, client_id)
            .await
        {
            Ok(ok) => ok,
            Err(error) => {
                if error.code() == RpcErrorCode::Fatal {
                    error!(
                        %error,
                        "fatal error fetching client state meta for client {client_id} on chain {}", self.chain_id
                    );

                    return Ok((vec![], noop()));
                } else {
                    error!(
                        %error,
                        "error fetching client state meta for client {client_id} on chain {}", self.chain_id
                    );
                }

                return Err(events
                    .into_iter()
                    .map(|(idxs, events)| {
                        (
                            idxs,
                            data(PluginMessage::new(
                                self.plugin_name(),
                                ModuleData::from(EventBatch { client_id, events }),
                            )),
                            self.plugin_name(),
                        )
                    })
                    .collect());
            }
        };

        let (idxs, events): (Vec<_>, Vec<_>) = events.into_iter().unzip();

        Ok((
            idxs.into_iter().flatten().collect::<Vec<_>>(),
            // REVIEW: This might need to be a seq depending on what the impl of the client update
            // plugin is
            conc(
                exact_target_heights
                    .into_iter()
                    .chain(min_target_height)
                    .map(|height| {
                        seq([
                            call(WaitForHeight {
                                chain_id: client_state_meta.counterparty_chain_id.clone(),
                                height,
                                finalized: true,
                            }),
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeTransactionBatchesWithUpdate {
                                    client_id,
                                    batches: events.clone(),
                                }),
                            )),
                        ])
                    }),
            ),
        ))
    }

    #[instrument(skip_all, fields(%proof_lens_client_id))]
    async fn query_proof_lens_client_state(
        &self,
        v: &VoyagerClient,
        proof_lens_client_id: ClientId,
    ) -> RpcResult<ClientState> {
        // info of the client on the target chain that will verify the storage proofs
        let proof_lens_client_info = v
            .client_info::<IbcUnion>(self.chain_id.clone(), proof_lens_client_id)
            .await?;

        debug!(
            %proof_lens_client_id,
            %proof_lens_client_info.client_type,
            %proof_lens_client_info.ibc_interface,
            %proof_lens_client_info.metadata,
            "proof lens client info"
        );

        let client_state = v
            .decode_client_state::<IbcUnion, ClientState>(
                proof_lens_client_info.client_type.clone(),
                proof_lens_client_info.ibc_interface.clone(),
                v.query_ibc_state(
                    self.chain_id.clone(),
                    QueryHeight::Latest,
                    ClientStatePath {
                        client_id: proof_lens_client_id,
                    },
                )
                .await?,
            )
            .await?;

        debug!(
            l2_chain_id = %client_state.l2_chain_id,
            l1_client_id = %client_state.l1_client_id,
            l2_client_id = %client_state.l2_client_id,
            l2_latest_height = %client_state.l2_latest_height,
            timestamp_offset = %client_state.timestamp_offset,
            "proof lens client state"
        );

        Ok(client_state)
    }

    #[instrument(skip_all, fields(%chain_id, %proof_height, path = %into_value(path)))]
    async fn query_and_encode_proof<P: IbcStorePathKey>(
        &self,
        v: &VoyagerClient,
        chain_id: &ChainId,
        proof_lens_client_state: &ClientState,
        proof_height: Height,
        path: &P,
    ) -> RpcResult<Bytes> {
        let proof = v
            .query_ibc_proof(
                chain_id.clone(),
                QueryHeight::Specific(proof_height),
                path.clone(),
            )
            .await?
            .into_result()?;

        if proof.proof_type != ProofType::Membership {
            return Err(RpcError::retryable_from_message(
                "proof is not a membership proof",
            ));
        }

        trace!(proof = %proof.proof, "proof");

        let l2_client_info = v
            .client_info::<IbcUnion>(
                v.client_state_meta::<IbcUnion>(
                    self.chain_id.clone(),
                    QueryHeight::Latest,
                    proof_lens_client_state.l1_client_id,
                )
                .await?
                .counterparty_chain_id,
                proof_lens_client_state.l2_client_id,
            )
            .await?;

        let encoded_proof = v
            .encode_proof::<IbcUnion>(
                l2_client_info.client_type.clone(),
                l2_client_info.ibc_interface.clone(),
                proof.proof,
            )
            .await?;

        trace!(%encoded_proof, "encoded proof");

        Ok(encoded_proof)
    }
}

fn commit(bytes: impl AsRef<[u8]>) -> H256 {
    keccak256(bytes)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn config_serde() {
        let config_json = json!({
          "chain_id": "union-devnet-1",
          "client_configs": {
            "min_batch_size": 1,
            "max_batch_size": 3,
            "max_wait_time": {
              "secs": 10,
              "nanos": 0
            }
          }
        });

        let config = serde_json::from_value::<Config>(config_json).unwrap();

        assert_eq!(
            config,
            Config {
                chain_id: ChainId::new("union-devnet-1"),
                client_configs: ClientConfigsSerde::Any(ClientConfig {
                    min_batch_size: 1,
                    max_batch_size: 3,
                    max_wait_time: Duration::from_secs(10)
                })
            }
        );
    }
}
