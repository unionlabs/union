use std::collections::VecDeque;

use enumorph::Enumorph;
use frunk::{hlist_pat, HList};
use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use macros::{apply, model};
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, HListTryFromIterator, UseAggregate},
    conc, fetch, queue_msg, HandleAggregate, Op, QueueError, QueueMessage,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, instrument};
use unionlabs::{
    hash::H256,
    ibc::core::{
        channel::{channel::Channel, packet::Packet},
        client::{height::Height, msg_update_client::MsgUpdateClient},
        connection::connection_end::ConnectionEnd,
    },
    ics24::{ChannelEndPath, ClientStatePath, ConnectionPath},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    traits::Member,
    QueryHeight,
};

use crate::{
    data::{
        ChannelMetadata, ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ClientInfo,
        ConnectionMetadata, ConnectionOpenAck, ConnectionOpenInit, ConnectionOpenTry, Data,
        DecodedClientStateMeta, IbcState, LatestHeight, OrderedHeaders, OrderedMsgUpdateClients,
        RawIbcProof, SendPacket, WriteAcknowledgement,
    },
    fetch::{
        compound::{fetch_client_state_meta, fetch_connection_from_channel_info},
        DecodeClientStateMeta, EncodeProof, FetchBlockRange, FetchClientInfo, FetchRawProof,
        FetchState,
    },
    json_rpc_error_to_queue_error,
    plugin::{ClientModuleClient, PluginModuleClient},
    top_level_identifiable_enum, Context, PluginMessage, VoyagerMessage,
};

#[apply(top_level_identifiable_enum)]
#[queue_msg]
#[derive(Enumorph)]
pub enum Aggregate<A = serde_json::Value> {
    // originally block
    FetchBlockRange(AggregateFetchBlockRange),

    AggregateMsgUpdateClientsFromOrderedHeaders(AggregateMsgUpdateClientsFromOrderedHeaders),
    // AggregateMsgUpdateClientFromEncodedHeader(AggregateMsgUpdateClientFromEncodedHeader),
    AggregateFetchClientInfoFromChannel(AggregateFetchClientFromChannel),
    AggregateFetchClientInfoFromConnection(AggregateFetchClientFromConnection),
    AggregateFetchConnectionFromChannel(AggregateFetchConnectionFromChannel),
    AggregateDecodeClientStateFromConnection(AggregateDecodeClientStateMetaFromConnection),

    AggregateFetchCounterpartyChannelAndConnection(AggregateFetchCounterpartyChannelAndConnection),

    AggregateFetchCounterpartyChannelAndConnectionFromSourceChannel(
        AggregateFetchCounterpartyChannelAndConnectionFromSourceChannel,
    ),

    AggregateEncodeStateProof(AggregateEncodeProof),

    // // originally relay
    // MsgConnectionOpenTry(AggregateMsgConnectionOpenTry),
    // MsgConnectionOpenAck(AggregateMsgConnectionOpenAck),
    // MsgConnectionOpenConfirm(AggregateMsgConnectionOpenConfirm),

    // MsgChannelOpenTry(AggregateMsgChannelOpenTry),
    // MsgChannelOpenAck(AggregateMsgChannelOpenAck),
    // MsgChannelOpenConfirm(AggregateMsgChannelOpenConfirm),

    // MsgRecvPacket(AggregateMsgRecvPacket),
    // MsgAckPacket(AggregateMsgAckPacket),
    // MsgTimeout(AggregateMsgTimeout),

    // // construct one of the above messages after a required client update
    // AggregateMsgAfterUpdate(AggregateMsgAfterUpdate),

    // MsgCreateClient(AggregateMsgCreateClient),

    // PacketTimeout(AggregatePacketTimeout),

    // // composite fetches
    // ReceiptPathProofFromChannelAndPort(AggregateFetchReceiptPathProofFromChannelAndPort),
    // ClientStateFromConnectionId(AggregateClientStateFromConnection),
    // ConnectionFetchFromChannelEnd(AggregateConnectionFetchFromChannelEnd),
    /// Aggregate that fetches the connection info from the channel, requeueing
    /// [`Self::AggregateMsgAfterUpdate`]
    // ChannelHandshakeMsgAfterUpdate(AggregateChannelHandshakeMsgAfterUpdate),

    // PacketUpdateClient(AggregatePacketMsgAfterUpdate),

    // WaitForConnectionOpen(AggregateWaitForConnectionOpen),
    // WaitForCounterpartyTrustedHeight(AggregateWaitForCounterpartyTrustedHeight),
    // WaitForTrustedHeight(AggregateWaitForTrustedHeight),
    // WaitForNextConnectionSequence(AggregateWaitForNextConnectionSequence),
    // WaitForNextClientSequence(AggregateWaitForNextClientSequence),
    // WaitForPacketReceipt(AggregateWaitForPacketReceipt),

    // FetchCounterpartyStateproof(AggregateFetchCounterpartyStateProof),

    // UpdateClient(AggregateUpdateClient),
    // UpdateClientFromHeight(AggregateUpdateClientFromHeight),

    // new
    AggregateDecodeClientStateMeta(AggregateDecodeClientStateMeta),

    Plugin(PluginMessage<A>),
}

impl<D: Member, F: Member, A: Member> HandleAggregate<VoyagerMessage<D, F, A>> for Aggregate<A> {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn handle(
        self,
        ctx: &Context,
        data: VecDeque<<VoyagerMessage<D, F, A> as QueueMessage>::Data>,
    ) -> Result<Op<VoyagerMessage<D, F, A>>, QueueError> {
        match self {
            Aggregate::FetchBlockRange(aggregate) => Ok(do_aggregate(aggregate, data)),

            Aggregate::AggregateMsgUpdateClientsFromOrderedHeaders(
                AggregateMsgUpdateClientsFromOrderedHeaders {
                    counterparty_client_id,
                },
            ) => {
                let Ok(
                    hlist_pat![
                        OrderedHeaders { headers },
                        ClientInfo {
                            client_type,
                            ibc_interface,
                            ..
                        }
                    ],
                ) = HListTryFromIterator::try_from_iter(data)
                else {
                    panic!("bad data")
                };

                let client_module =
                    ctx.client_module::<Value, Value, Value>(&client_type, &ibc_interface)?;

                Ok(queue_msg::data(OrderedMsgUpdateClients {
                    updates: stream::iter(headers.into_iter())
                        .then(|(meta, header)| {
                            client_module
                                .encode_header(header)
                                .map_ok(|encoded_header| {
                                    (
                                        meta,
                                        MsgUpdateClient {
                                            client_id: counterparty_client_id.clone(),
                                            client_message: encoded_header,
                                        },
                                    )
                                })
                                .map_err(json_rpc_error_to_queue_error)
                        })
                        .try_collect::<Vec<_>>()
                        .await?,
                }))
            }
            // Aggregate::AggregateMsgUpdateClientFromEncodedHeader(aggregate) => {
            //     Ok(do_aggregate(aggregate, data))
            // }
            Aggregate::AggregateFetchClientInfoFromChannel(aggregate) => {
                Ok(do_aggregate(aggregate, data))
            }
            Aggregate::AggregateFetchClientInfoFromConnection(aggregate) => {
                Ok(do_aggregate(aggregate, data))
            }
            Aggregate::AggregateFetchConnectionFromChannel(aggregate) => {
                Ok(do_aggregate(aggregate, data))
            }
            Aggregate::AggregateDecodeClientStateFromConnection(aggregate) => {
                Ok(do_aggregate(aggregate, data))
            }

            Aggregate::AggregateEncodeStateProof(aggregate) => Ok(do_aggregate(aggregate, data)),

            Aggregate::AggregateFetchCounterpartyChannelAndConnection(aggregate) => {
                Ok(do_aggregate(aggregate, data))
            }

            Aggregate::AggregateFetchCounterpartyChannelAndConnectionFromSourceChannel(
                aggregate,
            ) => Ok(do_aggregate(aggregate, data)),

            // Aggregate::MsgConnectionOpenTry(init) => {
            //     Ok(do_aggregate(id(self.chain_id, init), data))
            // }
            // Aggregate::MsgConnectionOpenAck(ack) => Ok(do_aggregate(id(self.chain_id, ack),
            // data)), Aggregate::MsgConnectionOpenConfirm(confirm) => {
            //     Ok(do_aggregate(id(self.chain_id, confirm), data))
            // }
            // Aggregate::MsgChannelOpenTry(try_) => Ok(do_aggregate(id(self.chain_id, try_),
            // data)), Aggregate::MsgChannelOpenAck(ack) =>
            // Ok(do_aggregate(id(self.chain_id, ack), data)),
            // Aggregate::MsgChannelOpenConfirm(confirm) => {
            //     Ok(do_aggregate(id(self.chain_id, confirm), data))
            // }
            // Aggregate::UpdateClient(update_client) => {
            //     Ok(do_aggregate(id(self.chain_id, update_client), data))
            // }
            // Aggregate::UpdateClientFromHeight(update_client) => {
            //     Ok(do_aggregate(id(self.chain_id, update_client), data))
            // }
            // Aggregate::MsgCreateClient(create_client) => {
            //     Ok(do_aggregate(id(self.chain_id, create_client), data))
            // }
            // Aggregate::AggregateMsgAfterUpdate(aggregate) => {
            //     Ok(do_aggregate(id(self.chain_id, aggregate), data))
            // }
            // Aggregate::ConnectionFetchFromChannelEnd(aggregate) => {
            //     Ok(do_aggregate(id(self.chain_id, aggregate), data))
            // }
            // Aggregate::ReceiptPathProofFromChannelAndPort(aggregate) => {
            //     Ok(do_aggregate(id(self.chain_id, aggregate), data))
            // }
            // Aggregate::ChannelHandshakeMsgAfterUpdate(channel_handshake_update_client) => Ok(
            //     do_aggregate(id(self.chain_id, channel_handshake_update_client), data),
            // ),
            // Aggregate::PacketUpdateClient(packet_update_client) => {
            //     Ok(do_aggregate(id(self.chain_id, packet_update_client), data))
            // }
            // Aggregate::MsgRecvPacket(recv_packet) => {
            //     Ok(do_aggregate(id(self.chain_id, recv_packet), data))
            // }
            // Aggregate::MsgAckPacket(ack_packet) => {
            //     Ok(do_aggregate(id(self.chain_id, ack_packet), data))
            // }
            // Aggregate::MsgTimeout(timeout_packet) => {
            //     Ok(do_aggregate(id(self.chain_id, timeout_packet), data))
            // }
            // Aggregate::PacketTimeout(timeout_packet) => {
            //     Ok(do_aggregate(id(self.chain_id, timeout_packet), data))
            // }
            // Aggregate::WaitForCounterpartyTrustedHeight(agg) => {
            //     Ok(do_aggregate(id(self.chain_id, agg), data))
            // }
            // Aggregate::WaitForTrustedHeight(agg) => Ok(do_aggregate(id(self.chain_id, agg),
            // data)), Aggregate::FetchCounterpartyStateproof(agg) => {
            //     Ok(do_aggregate(id(self.chain_id, agg), data))
            // }
            // Aggregate::ClientStateFromConnectionId(agg) => {
            //     Ok(do_aggregate(id(self.chain_id, agg), data))
            // }
            // Aggregate::WaitForConnectionOpen(agg) => Ok(do_aggregate(id(self.chain_id, agg),
            // data)), Aggregate::WaitForNextConnectionSequence(agg) => {
            //     Ok(do_aggregate(id(self.chain_id, agg), data))
            // }
            // Aggregate::WaitForNextClientSequence(agg) => {
            //     Ok(do_aggregate(id(self.chain_id, agg), data))
            // }
            // Aggregate::WaitForPacketReceipt(agg) => Ok(do_aggregate(id(self.chain_id, agg),
            // data)),
            Aggregate::AggregateDecodeClientStateMeta(agg) => Ok(do_aggregate(agg, data)),

            Aggregate::Plugin(PluginMessage { plugin, message }) => Ok(ctx
                .plugin(&plugin)?
                .handle_aggregate(message, data)
                .await
                .map_err(json_rpc_error_to_queue_error)?),
        }
    }
}

#[queue_msg]
pub struct AggregateFetchBlockRange {
    pub from_height: Height,
}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateFetchBlockRange
{
    type AggregatedData = HList![LatestHeight];

    fn aggregate(
        Self { from_height }: Self,
        hlist_pat![LatestHeight {
            chain_id,
            height: to_height
        }]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        assert!(to_height.revision_height > from_height.revision_height);

        fetch(FetchBlockRange {
            chain_id,
            from_height,
            to_height,
        })
    }
}

/// Required data: [`OrderedHeaders`] and [`ClientInfo`]
#[queue_msg]
pub struct AggregateMsgUpdateClientsFromOrderedHeaders {
    pub counterparty_client_id: ClientId,
}

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for AggregateMsgUpdateClientsFromOrderedHeaders
// {
//     type AggregatedData = HList![OrderedHeaders, ClientInfo];

//     fn aggregate(
//         Self {
//             counterparty_chain_id,
//             counterparty_client_id,
//         }: Self,
//         hlist_pat![OrderedHeaders { headers }, client_info]:
// Self::AggregatedData,     ) -> Op<VoyagerMessage<D, F, A>> {
//         seq(headers.into_iter().map(|header| {
//             aggregate(
//                 [fetch(FetchEncodeHeader {
//                     header,
//                     client_info: client_info.clone(),
//                 })],
//                 [],
//                 AggregateMsgUpdateClientFromEncodedHeader {
//                     counterparty_chain_id: counterparty_chain_id.clone(),
//                     counterparty_client_id: counterparty_client_id.clone(),
//                 },
//             )
//         }))
//     }
// }

// #[queue_msg]
// pub struct AggregateMsgUpdateClientFromEncodedHeader {
//     pub counterparty_chain_id: String,
//     pub counterparty_client_id: ClientId,
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for AggregateMsgUpdateClientFromEncodedHeader
// {
//     type AggregatedData = HList![EncodedHeader];

//     fn aggregate(
//         Self {
//             counterparty_chain_id,
//             counterparty_client_id,
//         }: Self,
//         hlist_pat![EncodedHeader { encoded_header }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         effect(WithChainId {
//             chain_id: counterparty_chain_id,
//             message: MsgUpdateClient {
//                 client_id: counterparty_client_id,
//                 client_message: encoded_header,
//             }
//             .into_super(),
//         })
//     }
// }

// REVIEW: Maybe add a "return channel" field? So that if both the
// channel/connection data is required then some rpc calls can be saved
#[queue_msg]
pub struct AggregateFetchClientFromChannel {
    pub fetch_type: InfoOrMeta,
}

#[model(no_serde)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfoOrMeta {
    Info,
    Meta,
    Both,
}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateFetchClientFromChannel
{
    type AggregatedData = HList![IbcState<ChannelEndPath>];

    fn aggregate(
        Self { fetch_type }: Self,
        hlist_pat![IbcState {
            chain_id,
            path: _,
            height,
            state: Channel {
                mut connection_hops,
                ..
            }
        }]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        aggregate(
            [fetch(FetchState {
                chain_id,
                path: ConnectionPath {
                    connection_id: connection_hops.pop().expect("empty connection hops?"),
                }
                .into(),
                at: QueryHeight::Specific(height),
            })],
            [],
            AggregateFetchClientFromConnection { fetch_type },
        )
    }
}

#[queue_msg]
pub struct AggregateFetchClientFromConnection {
    pub fetch_type: InfoOrMeta,
}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateFetchClientFromConnection
{
    type AggregatedData = HList![IbcState<ConnectionPath>];

    fn aggregate(
        Self { fetch_type }: Self,
        hlist_pat![IbcState {
            chain_id,
            state: ConnectionEnd { client_id, .. },
            height,
            ..
        }]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        let meta = || {
            fetch_client_state_meta(
                chain_id.clone(),
                client_id.clone(),
                QueryHeight::Specific(height),
            )
        };

        let info = || {
            fetch(FetchClientInfo {
                chain_id: chain_id.clone(),
                client_id: client_id.clone(),
            })
        };

        match fetch_type {
            InfoOrMeta::Info => info(),
            InfoOrMeta::Meta => meta(),
            InfoOrMeta::Both => conc([info(), meta()]),
        }
    }
}

// REVIEW: Maybe add a "return channel" field? So that if either/both of the
// channel/connection data is required then some rpc calls can be saved
#[queue_msg]
pub struct AggregateFetchConnectionFromChannel {}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateFetchConnectionFromChannel
{
    type AggregatedData = HList![IbcState<ChannelEndPath>];

    fn aggregate(
        AggregateFetchConnectionFromChannel {}: Self,
        hlist_pat![IbcState {
            chain_id,
            path: _,
            height,
            state: Channel {
                mut connection_hops,
                ..
            }
        }]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        fetch(FetchState {
            chain_id,
            path: ConnectionPath {
                connection_id: connection_hops.pop().expect("empty connection hops?"),
            }
            .into(),
            at: QueryHeight::Specific(height),
        })
    }
}

#[queue_msg]
pub struct AggregateEncodeProof {}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateEncodeProof
{
    type AggregatedData = HList![RawIbcProof, ClientInfo];

    fn aggregate(
        Self {}: Self,
        hlist_pat![raw_proof, client_info]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        fetch(EncodeProof {
            raw_proof,
            client_info,
        })
    }
}

#[queue_msg]
pub struct AggregateDecodeClientStateMeta {}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateDecodeClientStateMeta
{
    type AggregatedData = HList![IbcState<ClientStatePath>, ClientInfo];

    fn aggregate(
        Self {}: Self,
        hlist_pat![ibc_state, client_info]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        fetch(DecodeClientStateMeta {
            ibc_state,
            client_info,
        })
    }
}

#[queue_msg]
pub struct AggregateDecodeClientStateMetaFromConnection {}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateDecodeClientStateMetaFromConnection
{
    type AggregatedData = HList![IbcState<ConnectionPath>];

    fn aggregate(
        Self {}: Self,
        hlist_pat![ibc_state]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        aggregate(
            [
                fetch(FetchClientInfo {
                    chain_id: ibc_state.chain_id.clone(),
                    client_id: ibc_state.state.client_id.clone(),
                }),
                fetch(FetchState {
                    chain_id: ibc_state.chain_id,
                    // NOTE: Latest *MUST* be used here, as ibc_state.height is the height on the
                    // wrong chain
                    at: QueryHeight::Latest,
                    path: ClientStatePath {
                        client_id: ibc_state.state.client_id,
                    }
                    .into(),
                }),
            ],
            [],
            AggregateDecodeClientStateMeta {},
        )
    }
}

#[queue_msg]
pub struct AggregateMsgConnectionOpenTry {
    pub event_height: Height,
    pub event: ConnectionOpenInit,
}

#[queue_msg]
pub struct AggregateMsgConnectionOpenAck {
    pub event_height: Height,
    pub event: ConnectionOpenTry,
}

#[queue_msg]
pub struct AggregateMsgConnectionOpenConfirm {
    pub event_height: Height,
    pub event: ConnectionOpenAck,
}

#[queue_msg]
pub struct AggregateMsgChannelOpenTry {
    pub event_height: Height,
    pub event: ChannelOpenInit,
}

#[queue_msg]
pub struct AggregateMsgChannelOpenAck {
    pub event_height: Height,
    pub event: ChannelOpenTry,
}

#[queue_msg]
pub struct AggregateMsgChannelOpenConfirm {
    pub event_height: Height,
    pub event: ChannelOpenAck,
}

#[queue_msg]
pub struct AggregateMsgRecvPacket {
    pub event_height: Height,
    pub event: SendPacket,
}

#[queue_msg]
pub struct AggregateMsgAckPacket {
    pub event_height: Height,
    pub event: WriteAcknowledgement,
    // HACK: Need to pass the block hash through, figure out a better/cleaner way to do this
    // TODO: Replace with the ack directly?
    // TODO: Remove
    pub tx_hash: H256,
    pub counterparty_client_id: ClientId,
}

#[queue_msg]
pub struct AggregateMsgTimeout {
    // pub client_id: Hc::ClientId,
    // pub counterparty_client_id: Tr::ClientId,
    // pub counterparty_chain_idOf<Tr>,
    pub packet: Packet,
}

#[queue_msg]
pub struct AggregateConnectionFetchFromChannelEnd {
    pub at: Height,
}

#[queue_msg]
pub struct AggregateClientStateFromConnection {
    pub at: Height,
}

#[queue_msg]
pub struct AggregateFetchReceiptPathProofFromChannelAndPort {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

#[queue_msg]
pub struct AggregateChannelHandshakeMsgAfterUpdate {
    // Will be threaded through to the update msg
    pub event_height: Height,
    pub channel_handshake_event: ChannelHandshakeEvent,
}

#[queue_msg]
pub enum ChannelHandshakeEvent {
    Init(ChannelOpenInit),
    Try(ChannelOpenTry),
    Ack(ChannelOpenAck),
}

#[queue_msg]
pub struct AggregatePacketMsgAfterUpdate {
    // Will be threaded through to the update msg
    pub update_to: Height,
    pub event_height: Height,
    pub tx_hash: H256,
    pub packet_event: PacketEvent,
}

#[queue_msg]
pub enum PacketEvent {
    Send(SendPacket),
    WriteAck(WriteAcknowledgement),
}

#[queue_msg]
pub struct AggregatePacketTimeout {
    pub packet: Packet,
}

#[queue_msg]
pub struct AggregateWaitForPacketReceipt {
    pub packet: Packet,
}

#[queue_msg]
pub struct AggregateFetchCounterpartyStateProof {
    pub counterparty_client_id: ClientId,
    pub fetch: FetchRawProof,
}

#[queue_msg]
pub struct AggregateUpdateClient {
    pub client_id: ClientId,
}

#[queue_msg]
pub struct AggregateUpdateClientFromHeight {
    pub from_height: Height,
    pub client_id: ClientId,
}

#[queue_msg]
pub struct AggregateWaitForCounterpartyTrustedHeight {
    pub wait_for: Height,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[queue_msg]
pub struct AggregateWaitForConnectionOpen {
    pub connection_id: ConnectionId,
}

#[queue_msg]
pub struct AggregateWaitForNextConnectionSequence {
    pub sequence: u64,
}

#[queue_msg]
pub struct AggregateWaitForNextClientSequence {
    pub sequence: u64,
}

#[queue_msg]
pub struct AggregateWaitForTrustedHeight {
    pub client_id: ClientId,
    pub counterparty_chain_id: String,
    pub counterparty_client_id: ClientId,
}

/// Messages that will be re-queued after an update.
#[queue_msg]
pub enum AggregateMsgAfterUpdate {
    ConnectionOpenTry(AggregateMsgConnectionOpenTry),
    ConnectionOpenAck(AggregateMsgConnectionOpenAck),
    ConnectionOpenConfirm(AggregateMsgConnectionOpenConfirm),

    ChannelOpenTry(AggregateMsgChannelOpenTry),
    ChannelOpenAck(AggregateMsgChannelOpenAck),
    ChannelOpenConfirm(AggregateMsgChannelOpenConfirm),

    RecvPacket(AggregateMsgRecvPacket),
    AckPacket(AggregateMsgAckPacket),
    TimeoutPacket(AggregateMsgTimeout),
}

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateChannelHandshakeMsgAfterUpdate>
// {
//     type AggregatedData = HList![Identified<IbcState<ConnectionPath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateChannelHandshakeMsgAfterUpdate {
//                     channel_handshake_event,
//                     event_height,
//                 },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: self_chain_id,
//             t: IbcState {
//                 path: _,
//                 height: _,
//                 state: connection,
//                 ibc_interface: _,
//             },
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, self_chain_id);

//         let event_msg = match channel_handshake_event {
//             ChannelHandshakeEvent::Init(init) => {
//
// AggregateMsgAfterUpdate::ChannelOpenTry(AggregateMsgChannelOpenTry {
//                     event_height,
//                     event: init,
//                 })
//             }
//             ChannelHandshakeEvent::Try(try_) => {
//
// AggregateMsgAfterUpdate::ChannelOpenAck(AggregateMsgChannelOpenAck {
//                     event_height,
//                     event: try_,
//                 })
//             }
//             ChannelHandshakeEvent::Ack(ack) => {
//
// AggregateMsgAfterUpdate::ChannelOpenConfirm(AggregateMsgChannelOpenConfirm {
//                     event_height,
//                     event: ack,
//                 })
//             }
//         };

//         aggregate(
//             [mk_aggregate_wait_for_update(
//                 this_chain_id.clone(),
//                 connection.client_id,
//                 connection.counterparty.client_id,
//                 event_height,
//             )],
//             [],
//             id(this_chain_id, event_msg),
//         )
//     }
// }

// pub fn mk_aggregate_wait_for_update<D: Member, F: Member, A: Member>(
//     chain_id: String,
//     client_id: ClientId,
//     counterparty_client_id: ClientId,
//     wait_for: Height,
// ) -> Op<VoyagerMessage<D, F, A>> {
//     // fetch the latest client state, decode it, and then aggregate it down
// into a WaitForTrustedHeight message

//     aggregate(
//         [aggregate(
//             [fetch(id(
//                 chain_id.clone(),
//                 FetchState {
//                     at: QueryHeight::Latest,
//                     path: ClientStatePath {
//                         client_id: client_id.clone(),
//                     }
//                     .into(),
//                 },
//             ))],
//             [],
//             id(chain_id.clone(), AggregateDecodeClientState {}),
//         )],
//         [],
//         id(
//             chain_id,
//             AggregateWaitForCounterpartyTrustedHeight {
//                 wait_for,
//                 client_id,
//                 counterparty_client_id,
//             },
//         ),
//     )
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregatePacketMsgAfterUpdate>
// {
//     type AggregatedData = HList![Identified<IbcState<ConnectionPath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregatePacketMsgAfterUpdate {
//                     update_to,
//                     event_height,
//                     tx_hash,
//                     packet_event,
//                 },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: self_chain_id,
//             t: IbcState {
//                 path: _,
//                 height: _,
//                 state: connection,
//                 ibc_interface: _,
//             },
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, self_chain_id);

//         let event = match packet_event {
//             PacketEvent::Send(send) =>
// Aggregate::from(AggregateMsgAfterUpdate::RecvPacket(
// AggregateMsgRecvPacket {                     event_height,
//                     event: send,
//                 },
//             )),
//             PacketEvent::WriteAck(recv) => {
//
// Aggregate::from(AggregateMsgAfterUpdate::AckPacket(AggregateMsgAckPacket {
//                     event_height,
//                     event: recv,
//                     tx_hash,
//                     counterparty_client_id:
// connection.counterparty.client_id.clone(),                 }))
//             }
//         };

//         aggregate(
//             [aggregate(
//                 [fetch(id(
//                     this_chain_id.clone().clone(),
//                     FetchState {
//                         at: QueryHeight::Latest,
//                         path: ClientStatePath {
//                             client_id: connection.client_id.clone(),
//                         }
//                         .into(),
//                     },
//                 ))],
//                 [],
//                 id(
//                     this_chain_id.clone(),
//                     AggregateWaitForCounterpartyTrustedHeight {
//                         wait_for: update_to,
//                         client_id: connection.client_id.clone(),
//                         counterparty_client_id:
// connection.counterparty.client_id.clone(),                     },
//                 ),
//             )],
//             [],
//             id(this_chain_id, event),
//         )
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregatePacketTimeout>
// {
//     type AggregatedData = HList![
//         Identified<DecodedClientStateMeta>,
//         Identified<IbcState<ConnectionPath>>
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregatePacketTimeout { packet },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: client_state_chain_id,
//                 t: DecodedClientStateMeta {
//                     path: ClientStatePath { client_id },
//                     height: _,
//                     state: client_state,
//                 },
//             },
//             Identified {
//                 chain_id: connection_chain_id,
//                 t: IbcState {
//                     path: _,
//                     height: _,
//                     state: connection,
//                     ibc_interface: _,
//                 },
//             }
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, client_state_chain_id);
//         assert_eq!(this_chain_id, connection_chain_id);
//         assert_eq!(client_id, connection.client_id);

//         let counterparty_chain_id = client_state.chain_id;

//         let aggregate_timeout = id(
//             counterparty_chain_id.clone(),
//             AggregateMsgAfterUpdate::TimeoutPacket(AggregateMsgTimeout {
//                 packet: packet.clone(),
//             }),
//         );

//         race(
//             [
//                 Some(aggregate(
//                     [fetch(id(
//                         counterparty_chain_id.clone(),
//                         FetchState {
//                             at: QueryHeight::Latest,
//                             path: ReceiptPath {
//                                 port_id: packet.destination_port.clone(),
//                                 channel_id:
// packet.destination_channel.clone(),                                 sequence:
// packet.sequence,                             }
//                             .into(),
//                         },
//                     ))],
//                     [],
//                     id(
//                         counterparty_chain_id.clone(),
//                         AggregateWaitForPacketReceipt {
//                             packet: packet.clone(),
//                         },
//                     ),
//                 )),
//                 (packet.timeout_height != Height::default()).then(|| {
//                     aggregate(
//                         [
//                             wait(id(
//                                 counterparty_chain_id.clone(),
//                                 WaitForHeight {
//                                     height: packet.timeout_height,
//                                 },
//                             )),
//                             // we bypass
// `AggregateWaitForCounterpartyTrustedHeight` here because
// // we already have the client state                             wait(id(
//                                 this_chain_id.clone(),
//                                 WaitForTrustedHeight {
//                                     height: packet.timeout_height.into(),
//                                     client_id: client_id.clone(),
//                                     counterparty_client_id: connection
//                                         .counterparty
//                                         .client_id
//                                         .clone(),
//                                     counterparty_chain_id:
// counterparty_chain_id.clone(),                                 },
//                             )),
//                         ],
//                         [],
//                         aggregate_timeout.clone(),
//                     )
//                 }),
//                 (packet.timeout_timestamp != 0).then(|| {
//                     aggregate(
//                         [aggregate(
//                             // `WaitForTimestamp` returns the latest height
// if the timestamp has                             // been hit (note that this
// will be changed to return the height of the                             //
// timestamp eventually, which is why we don't use
// // `seq(wait(timestamp), fetch(latest_height))`)
// [wait(id(                                 counterparty_chain_id.clone(),
//                                 WaitForTimestamp {
//                                     timestamp: i64::try_from(
//                                         // TODO: normalizes for voyager that
//                                         // expects seconds, we may just move
// to                                         // nanoseconds for everything to
// avoid                                         // any friction in the
// interface                                         // Add one second as we
// truncate the nanos.
// (packet.timeout_timestamp / (1e9 as u64)) + 1,
// )
// .map_err(json_rpc_error_to_queue_error)?,                                 },
//                             ))],
//                             [],
//                             id(
//                                 this_chain_id.clone(),
//                                 AggregateWaitForTrustedHeight {
//                                     client_id: client_id.clone(),
//                                     counterparty_client_id: connection
//                                         .counterparty
//                                         .client_id
//                                         .clone(),
//                                     counterparty_chain_id:
// counterparty_chain_id.clone(),                                 },
//                             ),
//                         )],
//                         [],
//                         aggregate_timeout.clone(),
//                     )
//                 }),
//             ]
//             .into_iter()
//             .flatten(),
//         )
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateWaitForTrustedHeight>
// {
//     type AggregatedData = HList![Identified<LatestHeight>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateWaitForTrustedHeight {
//                     client_id,
//                     counterparty_chain_id,
//                     counterparty_client_id,
//                 },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: latest_height_chain_id,
//             t: LatestHeight(height),
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(latest_height_chain_id, counterparty_chain_id);

//         wait(id(
//             this_chain_id,
//             WaitForTrustedHeight {
//                 height,
//                 client_id,
//                 counterparty_client_id,
//                 counterparty_chain_id,
//             },
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateConnectionFetchFromChannelEnd>
// {
//     type AggregatedData = HList![Identified<IbcState<ChannelEndPath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregateConnectionFetchFromChannelEnd { at },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: self_chain_id,
//             t: IbcState {
//                 path: _,
//                 height: _,
//                 state: channel,
//                 ibc_interface: _,
//             },
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, self_chain_id);

//         fetch(id(
//             this_chain_id,
//             FetchState {
//                 at: QueryHeight::Specific(at),
//                 path: ConnectionPath {
//                     connection_id: channel.connection_hops[0].clone(),
//                 }
//                 .into(),
//             },
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateClientStateFromConnection>
// {
//     type AggregatedData = HList![Identified<IbcState<ConnectionPath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregateClientStateFromConnection { at },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: self_chain_id,
//             t: IbcState {
//                 path: _,
//                 height: _,
//                 state: connection,
//                 ibc_interface: _,
//             },
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, self_chain_id);

//         fetch(id(
//             this_chain_id,
//             FetchState {
//                 at: QueryHeight::Specific(at),
//                 path: ClientStatePath {
//                     client_id: connection.client_id,
//                 }
//                 .into(),
//             },
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateUpdateClient>
// {
//     type AggregatedData = HList![Identified<DecodedClientStateMeta>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregateUpdateClient { client_id },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: self_chain_id,
//             t: DecodedClientStateMeta {
//                 path: ClientStatePath {
//                     client_id: trusted_client_state_client_id
//                 },
//                 height: _trusted_client_state_fetched_at_height,
//                 state: trusted_client_state,
//             },
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         let counterparty_chain_id = trusted_client_state.chain_id;

//         assert_eq!(trusted_client_state_client_id, client_id);
//         assert_eq!(this_chain_id, self_chain_id);

//         aggregate(
//             [fetch(id(counterparty_chain_id, FetchLatestHeight {}))],
//             [],
//             id(
//                 this_chain_id,
//                 AggregateUpdateClientFromHeight {
//                     from_height: trusted_client_state.height,
//                     client_id,
//                 },
//             ),
//         )
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateUpdateClientFromHeight>
// {
//     type AggregatedData = HList![Identified<LatestHeight>,];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateUpdateClientFromHeight {
//                     client_id,
//                     from_height,
//                 },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: counterparty_chain_id,
//             t: LatestHeight(counterparty_latest_height),
//         },]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         fetch(id(
//             counterparty_chain_id,
//             FetchUpdateHeaders {
//                 counterparty_client_id: client_id,
//                 counterparty_chain_id: this_chain_id,
//                 update_from: from_height,
//                 update_to: counterparty_latest_height,
//             },
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateWaitForCounterpartyTrustedHeight>
// {
//     type AggregatedData = HList![Identified<DecodedClientStateMeta>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateWaitForCounterpartyTrustedHeight {
//                     wait_for,
//                     client_id,
//                     counterparty_client_id,
//                 },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: trusted_client_state_chain_id,
//             t: DecodedClientStateMeta {
//                 path: ClientStatePath {
//                     client_id: trusted_client_state_client_id
//                 },
//                 height: _trusted_client_state_fetched_at_height,
//                 state: trusted_client_state,
//             },
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(trusted_client_state_client_id, client_id);
//         assert_eq!(trusted_client_state_chain_id, this_chain_id);

//         let counterparty_chain_id = trusted_client_state.chain_id;

//         wait(id(
//             counterparty_chain_id,
//             WaitForTrustedHeight {
//                 height: wait_for,
//                 client_id: counterparty_client_id,
//                 counterparty_client_id: client_id,
//                 counterparty_chain_id: this_chain_id,
//             },
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgAfterUpdate>
// {
//     type AggregatedData = HList![Identified<DecodedClientStateMeta>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: msg_to_aggregate,
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: self_chain_id,
//             t: DecodedClientStateMeta {
//                 path: ClientStatePath {
//                     client_id: trusted_client_state_client_id
//                 },
//                 height: trusted_client_state_fetched_at_height,
//                 state: trusted_client_state,
//             },
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, self_chain_id);

//         match msg_to_aggregate {
//
// AggregateMsgAfterUpdate::ConnectionOpenTry(AggregateMsgConnectionOpenTry {
//                 event_height,
//                 event,
//             }) => {
//                 let consensus_state_height =
// trusted_client_state_fetched_at_height;

//                 assert_eq!(
//                     consensus_state_height.revision_number,
// event_height.revision_number,                     "consensus state height
// `{consensus_state_height}` and event height \
// `{event_height}` have different revision numbers",                 );

//                 assert!(
//                     consensus_state_height.revision_height >=
// event_height.revision_height,                     "{} < {}",
//                     consensus_state_height.revision_height,
//                     event_height.revision_height
//                 );

//                 let trusted_client_state_height =
// trusted_client_state.height();

//                 aggregate(
//                     [
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ClientStatePath {
//                                     client_id: event.client_id.clone(),
//                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ClientConsensusStatePath {
//                                     client_id: event.client_id.clone(),
//                                     height: trusted_client_state_height,
//                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ConnectionPath {
//                                     connection_id:
// event.connection_id.clone(),                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchState {
//                                 at:
// QueryHeight::Specific(trusted_client_state_fetched_at_height),
// path: ConnectionPath {                                     connection_id:
// event.connection_id.clone(),                                 }
//                                 .into(),
//                             },
//                         )),
//                     ],
//                     [id(
//                         this_chain_id.clone(),
//                         IbcState::<ClientStatePath> {
//                             path: ClientStatePath {
//                                 client_id: trusted_client_state_client_id,
//                             },
//                             height: trusted_client_state_fetched_at_height,
//                             state: trusted_client_state,
//                         },
//                     )
//                     .into()],
//                     id(
//                         this_chain_id,
//                         AggregateMsgConnectionOpenTry {
//                             event_height,
//                             event,
//                         },
//                     ),
//                 )
//             }
//
// AggregateMsgAfterUpdate::ConnectionOpenAck(AggregateMsgConnectionOpenAck {
//                 event_height,
//                 event,
//             }) => {
//                 let consensus_state_height =
// trusted_client_state_fetched_at_height;

//                 assert_eq!(
//                     consensus_state_height.revision_number,
// event_height.revision_number,                     "consensus state height
// `{consensus_state_height}` and event height \
// `{event_height}` have different revision numbers",                 );

//                 assert!(
//                     consensus_state_height.revision_height >=
// event_height.revision_height,                     "{} < {}",
//                     consensus_state_height.revision_height,
//                     event_height.revision_height
//                 );

//                 let trusted_client_state_height =
// trusted_client_state.height();

//                 aggregate(
//                     [
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ClientStatePath {
//                                     client_id: event.client_id.clone(),
//                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ClientConsensusStatePath {
//                                     client_id: event.client_id.clone(),
//                                     height: trusted_client_state_height,
//                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ConnectionPath {
//                                     connection_id:
// event.connection_id.clone(),                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchState {
//                                 at:
// QueryHeight::Specific(trusted_client_state_fetched_at_height),
// path: ConnectionPath {                                     connection_id:
// event.connection_id.clone(),                                 }
//                                 .into(),
//                             },
//                         )),
//                     ],
//                     [id(
//                         this_chain_id.clone(),
//                         IbcState {
//                             path: ClientStatePath {
//                                 client_id: trusted_client_state_client_id,
//                             },
//                             height: trusted_client_state_fetched_at_height,
//                             state: trusted_client_state,
//                         },
//                     )
//                     .into()],
//                     id(
//                         this_chain_id,
//                         AggregateMsgConnectionOpenAck {
//                             event_height,
//                             event,
//                         },
//                     ),
//                 )
//             }
//
// AggregateMsgAfterUpdate::ConnectionOpenConfirm(AggregateMsgConnectionOpenConfirm
// {                 event_height,
//                 event,
//             }) => {
//                 let consensus_state_height =
// trusted_client_state_fetched_at_height;

//                 assert_eq!(
//                     consensus_state_height.revision_number,
// event_height.revision_number,                     "consensus state height
// `{consensus_state_height}` and event height \
// `{event_height}` have different revision numbers",                 );

//                 assert!(
//                     consensus_state_height.revision_height >=
// event_height.revision_height,                     "{} < {}",
//                     consensus_state_height.revision_height,
//                     event_height.revision_height
//                 );

//                 aggregate(
//                     [fetch(id(
//                         this_chain_id.clone(),
//                         FetchProof {
//                             at: trusted_client_state_fetched_at_height,
//                             path: ConnectionPath {
//                                 connection_id: event.connection_id.clone(),
//                             }
//                             .into(),
//                         },
//                     ))],
//                     [id(
//                         this_chain_id.clone(),
//                         IbcState {
//                             path: ClientStatePath {
//                                 client_id: trusted_client_state_client_id,
//                             },
//                             height: trusted_client_state_fetched_at_height,
//                             state: trusted_client_state,
//                         },
//                     )
//                     .into()],
//                     id(
//                         this_chain_id,
//                         AggregateMsgConnectionOpenConfirm {
//                             event_height,
//                             event,
//                         },
//                     ),
//                 )
//             }
//
// AggregateMsgAfterUpdate::ChannelOpenTry(AggregateMsgChannelOpenTry {
//                 event_height,
//                 event,
//             }) => {
//                 let consensus_state_height =
// trusted_client_state_fetched_at_height;

//                 assert_eq!(
//                     consensus_state_height.revision_number,
// event_height.revision_number,                     "consensus state height
// `{consensus_state_height}` and event height \
// `{event_height}` have different revision numbers",                 );

//                 assert!(
//                     consensus_state_height.revision_height >=
// event_height.revision_height,                     "{} < {}",
//                     consensus_state_height.revision_height,
//                     event_height.revision_height
//                 );

//                 aggregate(
//                     [
//                         aggregate(
//                             [fetch(id(
//                                 this_chain_id.clone(),
//                                 FetchState {
//                                     at: QueryHeight::Specific(
//
// trusted_client_state_fetched_at_height,
// ),                                     path: ChannelEndPath {
//                                         port_id: event.port_id.clone(),
//                                         channel_id: event.channel_id.clone(),
//                                     }
//                                     .into(),
//                                 },
//                             ))],
//                             [],
//                             id(
//                                 this_chain_id.clone(),
//                                 AggregateConnectionFetchFromChannelEnd {
//                                     at:
// trusted_client_state_fetched_at_height,                                 },
//                             ),
//                         ),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ChannelEndPath {
//                                     port_id: event.port_id.clone(),
//                                     channel_id: event.channel_id.clone(),
//                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchState {
//                                 at:
// QueryHeight::Specific(trusted_client_state_fetched_at_height),
// path: ChannelEndPath {                                     port_id:
// event.port_id.clone(),                                     channel_id:
// event.channel_id.clone(),                                 }
//                                 .into(),
//                             },
//                         )),
//                     ],
//                     [id(
//                         this_chain_id.clone(),
//                         IbcState {
//                             path: ClientStatePath {
//                                 client_id: trusted_client_state_client_id,
//                             },
//                             height: trusted_client_state_fetched_at_height,
//                             state: trusted_client_state,
//                         },
//                     )
//                     .into()],
//                     id(
//                         this_chain_id,
//                         AggregateMsgChannelOpenTry {
//                             event_height,
//                             event,
//                         },
//                     ),
//                 )
//             }
//
// AggregateMsgAfterUpdate::ChannelOpenAck(AggregateMsgChannelOpenAck {
//                 event_height,
//                 event,
//             }) => {
//                 let consensus_state_height =
// trusted_client_state_fetched_at_height;

//                 assert_eq!(
//                     consensus_state_height.revision_number,
// event_height.revision_number,                     "{consensus_state_height},
// {event_height}",                 );

//                 assert!(
//                     consensus_state_height.revision_height >=
// event_height.revision_height,                     "{} < {}",
//                     consensus_state_height.revision_height,
//                     event_height.revision_height
//                 );

//                 aggregate(
//                     [
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ChannelEndPath {
//                                     port_id: event.port_id.clone(),
//                                     channel_id: event.channel_id.clone(),
//                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchState {
//                                 at:
// QueryHeight::Specific(trusted_client_state_fetched_at_height),
// path: ChannelEndPath {                                     port_id:
// event.port_id.clone(),                                     channel_id:
// event.channel_id.clone(),                                 }
//                                 .into(),
//                             },
//                         )),
//                     ],
//                     [id(
//                         this_chain_id.clone(),
//                         IbcState {
//                             path: ClientStatePath {
//                                 client_id: trusted_client_state_client_id,
//                             },
//                             height: trusted_client_state_fetched_at_height,
//                             state: trusted_client_state,
//                         }
//                         .into(),
//                     )
//                     .into()],
//                     id(
//                         this_chain_id,
//                         AggregateMsgChannelOpenAck {
//                             event_height,
//                             event,
//                         },
//                     ),
//                 )
//             }
//
// AggregateMsgAfterUpdate::ChannelOpenConfirm(AggregateMsgChannelOpenConfirm {
//                 event_height,
//                 event,
//             }) => {
//                 let consensus_state_height =
// trusted_client_state_fetched_at_height;

//                 assert_eq!(
//                     consensus_state_height.revision_number,
// event_height.revision_number,                     "{consensus_state_height},
// {event_height}",                 );

//                 assert!(
//                     consensus_state_height.revision_height >=
// event_height.revision_height,                     "{} < {}",
//                     consensus_state_height.revision_height,
//                     event_height.revision_height
//                 );

//                 aggregate(
//                     [
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ChannelEndPath {
//                                     port_id: event.port_id.clone(),
//                                     channel_id: event.channel_id.clone(),
//                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchState {
//                                 at:
// QueryHeight::Specific(trusted_client_state_fetched_at_height),
// path: ChannelEndPath {                                     port_id:
// event.port_id.clone(),                                     channel_id:
// event.channel_id.clone(),                                 }
//                                 .into(),
//                             },
//                         )),
//                     ],
//                     [id(
//                         this_chain_id.clone(),
//                         IbcState {
//                             path: ClientStatePath {
//                                 client_id: trusted_client_state_client_id,
//                             },
//                             height: trusted_client_state_fetched_at_height,
//                             state: trusted_client_state,
//                         },
//                     )
//                     .into()],
//                     id(
//                         this_chain_id,
//                         AggregateMsgChannelOpenConfirm {
//                             event_height,
//                             event,
//                         },
//                     ),
//                 )
//             }
//             AggregateMsgAfterUpdate::RecvPacket(AggregateMsgRecvPacket {
//                 event_height,
//                 event,
//             }) => aggregate(
//                 [fetch(id(
//                     this_chain_id.clone(),
//                     FetchProof {
//                         at: trusted_client_state_fetched_at_height,
//                         path: CommitmentPath {
//                             port_id: event.packet_src_port.clone(),
//                             channel_id: event.packet_src_channel.clone(),
//                             sequence: event.packet_sequence,
//                         }
//                         .into(),
//                     },
//                 ))],
//                 [id(
//                     this_chain_id.clone(),
//                     IbcState {
//                         path: ClientStatePath {
//                             client_id: trusted_client_state_client_id,
//                         },
//                         height: trusted_client_state_fetched_at_height,
//                         state: trusted_client_state,
//                     },
//                 )
//                 .into()],
//                 id(
//                     this_chain_id,
//                     AggregateMsgRecvPacket {
//                         event_height,
//                         event,
//                     },
//                 ),
//             ),
//             AggregateMsgAfterUpdate::AckPacket(AggregateMsgAckPacket {
//                 event_height,
//                 event,
//                 tx_hash,
//                 counterparty_client_id,
//             }) => aggregate(
//                 [fetch(id(
//                     this_chain_id.clone(),
//                     FetchProof {
//                         at: trusted_client_state_fetched_at_height,
//                         path: AcknowledgementPath {
//                             port_id: event.packet_dst_port.clone(),
//                             channel_id: event.packet_dst_channel.clone(),
//                             sequence: event.packet_sequence,
//                         }
//                         .into(),
//                     },
//                 ))],
//                 [id(
//                     this_chain_id.clone(),
//                     IbcState {
//                         path: ClientStatePath {
//                             client_id: trusted_client_state_client_id,
//                         },
//                         height: trusted_client_state_fetched_at_height,
//                         state: trusted_client_state,
//                     },
//                 )
//                 .into()],
//                 id(
//                     this_chain_id,
//                     AggregateMsgAckPacket {
//                         event_height,
//                         event,
//                         tx_hash,
//                         counterparty_client_id,
//                     },
//                 ),
//             ),
//             AggregateMsgAfterUpdate::TimeoutPacket(AggregateMsgTimeout {
// packet }) => {                 aggregate(
//                     [
//                         // NOTE: Use this when we support ordered packets
//                         //     aggregate(
//                         //     // fetch the packet nonexistence proof from
// the counterparty                         //     [fetch(id(
//                         //         this_chain_id.clone(),
//                         //         FetchState {
//                         //             at:
//                         //
// QueryHeight::Specific(trusted_client_state_fetched_at_height),
// //             path: NextSequenceRecvPath {                         //
// port_id: packet.destination_port.clone(),                         //
// channel_id: packet.destination_channel.clone(),                         //
// }                         //             .into(),
//                         //         },
//                         //     ))],
//                         //     [],
//                         //     id(
//                         //         this_chain_id.clone(),
//                         //
// AggregateFetchReceiptPathProofFromChannelAndPort {                         //
// port_id: packet.destination_port.clone(),                         //
// channel_id: packet.destination_channel.clone(),                         //
//                         //         },
//                         //     ),
//                         // )
//                         fetch(id(
//                             this_chain_id.clone(),
//                             FetchProof {
//                                 at: trusted_client_state_fetched_at_height,
//                                 path: ReceiptPath {
//                                     port_id: packet.destination_port.clone(),
//                                     channel_id:
// packet.destination_channel.clone(),
// sequence: packet.sequence,                                 }
//                                 .into(),
//                             },
//                         )),
//                         fetch(id(
//                             this_chain_id,
//                             FetchState {
//                                 at:
// QueryHeight::Specific(trusted_client_state_fetched_at_height),
// path: ReceiptPath {                                     port_id:
// packet.destination_port.clone(),
// channel_id: packet.destination_channel.clone(),
// sequence: packet.sequence,                                 }
//                                 .into(),
//                             },
//                         )),
//                     ],
//                     [],
//                     id(
//                         trusted_client_state.chain_id(),
//                         AggregateMsgTimeout { packet },
//                     ),
//                 )
//             }
//         }
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgConnectionOpenTry>
// {
//     type AggregatedData = HList![
//         Identified<IbcState<ClientStatePath>>,
//         Identified<IbcProof<ClientStatePath>>,
//         Identified<IbcProof<ClientConsensusStatePath>>,
//         Identified<IbcProof<ConnectionPath>>,
//         Identified<IbcState<ConnectionPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateMsgConnectionOpenTry {
//                     event_height: trusted_height,
//                     event,
//                 },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: trusted_client_state_chain_id,
//                 t: IbcState {
//                     path: ClientStatePath {
//                         client_id: _trusted_client_state_client_id
//                     },
//                     height: _trusted_client_state_fetched_at_height,
//                     state: trusted_client_state,
//                     ibc_interface: _
//                 },
//             },
//             Identified {
//                 chain_id: client_state_proof_chain_id,
//                 t: IbcProof {
//                     height: client_state_proof_height,
//                     proof: client_state_proof,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: consensus_state_proof_chain_id,
//                 t: IbcProof {
//                     height: consensus_state_proof_height,
//                     proof: consensus_state_proof,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: connection_proof_chain_id,
//                 t: IbcProof {
//                     height: connection_proof_height,
//                     proof: connection_proof,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: connection_end_chain_id,
//                 t: IbcState {
//                     path: _,
//                     height: _,
//                     state: connection_end,
//                     ibc_interface: _
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert!(consensus_state_proof_height.revision_height >=
// trusted_height.revision_height);         assert!(client_state_proof_height.
// revision_height >= trusted_height.revision_height);

//         let counterparty_chain_id = trusted_client_state.chain_id();

//         assert_eq!(trusted_client_state_chain_id, this_chain_id);

//         assert_eq!(client_state_proof_chain_id, this_chain_id);
//         assert_eq!(consensus_state_proof_chain_id, this_chain_id);
//         assert_eq!(connection_proof_chain_id, this_chain_id);
//         assert_eq!(connection_end_chain_id, this_chain_id);

//         let consensus_height = trusted_client_state.height();

//         effect(id(
//             counterparty_chain_id,
//             MsgConnectionOpenTryData(MsgConnectionOpenTry {
//                 client_id: event.counterparty_client_id,
//                 client_state: trusted_client_state,
//                 counterparty: connection::counterparty::Counterparty {
//                     client_id: event.client_id,
//                     connection_id: event.connection_id,
//                     prefix: MerklePrefix {
//                         // TODO: Make configurable?
//                         key_prefix: b"ibc".to_vec(),
//                     },
//                 },
//                 // TODO: Make configurable?
//                 delay_period: DELAY_PERIOD,
//                 counterparty_versions: connection_end.versions,
//                 proof_height: connection_proof_height,
//                 proof_init: connection_proof,
//                 proof_client: client_state_proof,
//                 proof_consensus: consensus_state_proof,
//                 consensus_height,
//             }),
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgConnectionOpenAck>
// {
//     type AggregatedData = HList![
//         Identified<IbcState<ClientStatePath>>,
//         Identified<IbcProof<ClientStatePath>>,
//         Identified<IbcProof<ClientConsensusStatePath>>,
//         Identified<IbcProof<ConnectionPath>>,
//         Identified<IbcState<ConnectionPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateMsgConnectionOpenAck {
//                     event_height: trusted_height,
//                     event,
//                 },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: trusted_client_state_chain_id,
//                 t: IbcState {
//                     path: ClientStatePath {
//                         client_id: _trusted_client_state_client_id
//                     },
//                     height: _trusted_client_state_fetched_at_height,
//                     state: trusted_client_state,
//                     ibc_interface: _
//                 },
//             },
//             Identified {
//                 chain_id: client_state_proof_chain_id,
//                 t: IbcProof {
//                     height: client_state_proof_height,
//                     proof: client_state_proof,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: consensus_state_proof_chain_id,
//                 t: IbcProof {
//                     height: consensus_state_proof_height,
//                     proof: consensus_state_proof,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: connection_proof_chain_id,
//                 t: IbcProof {
//                     height: connection_proof_height,
//                     proof: connection_proof,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: connection_end_chain_id,
//                 t: IbcState {
//                     path: _,
//                     height: _,
//                     state: connection_end,
//                     ibc_interface: _
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert!(consensus_state_proof_height.revision_height >=
// trusted_height.revision_height);         assert!(client_state_proof_height.
// revision_height >= trusted_height.revision_height);

//         let counterparty_chain_id = trusted_client_state.chain_id();

//         assert_eq!(trusted_client_state_chain_id, this_chain_id);
//         assert_eq!(client_state_proof_chain_id, this_chain_id);
//         assert_eq!(consensus_state_proof_chain_id, this_chain_id);
//         assert_eq!(connection_proof_chain_id, this_chain_id);
//         assert_eq!(connection_end_chain_id, this_chain_id);

//         let consensus_height = trusted_client_state.height();

//         effect(id(
//             counterparty_chain_id,
//             MsgConnectionOpenAckData(MsgConnectionOpenAck {
//                 connection_id: event.counterparty_connection_id,
//                 counterparty_connection_id: event.connection_id,
//                 // TODO: Figure out a way to not panic here, likely by
// encoding this invariant into                 // the type somehow
//                 version: connection_end.versions[0].clone(),
//                 client_state: trusted_client_state,
//                 proof_height: connection_proof_height.into(),
//                 proof_try: connection_proof,
//                 proof_client: client_state_proof,
//                 proof_consensus: consensus_state_proof,
//                 consensus_height: consensus_height.into(),
//             }),
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgConnectionOpenConfirm>
// {
//     type AggregatedData = HList![
//         Identified<IbcState<ClientStatePath>>,
//         Identified<IbcProof<ConnectionPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateMsgConnectionOpenConfirm {
//                     event_height: _,
//                     event,
//                 },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: trusted_client_state_chain_id,
//                 t: IbcState {
//                     path: ClientStatePath {
//                         client_id: _trusted_client_state_client_id
//                     },
//                     height: _trusted_client_state_fetched_at_height,
//                     state: trusted_client_state,
//                     ibc_interface: _
//                 },
//             },
//             Identified {
//                 chain_id: connection_proof_chain_id,
//                 t: IbcProof {
//                     height: connection_proof_height,
//                     proof: connection_proof,
//                     path: _,
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         let counterparty_chain_id = trusted_client_state.chain_id();

//         assert_eq!(trusted_client_state_chain_id, this_chain_id);
//         assert_eq!(connection_proof_chain_id, this_chain_id);

//         effect(id(
//             counterparty_chain_id,
//             MsgConnectionOpenConfirmData(MsgConnectionOpenConfirm {
//                 connection_id: event.counterparty_connection_id,
//                 proof_height: connection_proof_height,
//                 proof_ack: connection_proof,
//             }),
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgChannelOpenTry>
// {
//     type AggregatedData = HList![
//         Identified<IbcState<ClientStatePath>>,
//         Identified<IbcProof<ChannelEndPath>>,
//         Identified<IbcState<ConnectionPath>>,
//         Identified<IbcState<ChannelEndPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateMsgChannelOpenTry {
//                     event_height: _,
//                     event,
//                 },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: trusted_client_state_chain_id,
//                 t: IbcState {
//                     path: ClientStatePath {
//                         client_id: _trusted_client_state_client_id
//                     },
//                     height: _trusted_client_state_fetched_at_height,
//                     state: trusted_client_state,
//                     ibc_interface: _
//                 },
//             },
//             Identified {
//                 chain_id: channel_proof_chain_id,
//                 t: IbcProof {
//                     proof: channel_proof,
//                     height: channel_proof_height,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: _connection_end_chain_id,
//                 t: IbcState {
//                     path: _,
//                     height: _,
//                     state: connection,
//                     ibc_interface: _
//                 },
//             },
//             Identified {
//                 chain_id: _channel_end_chain_id,
//                 t: IbcState {
//                     path: _,
//                     height: _,
//                     state: channel,
//                     ibc_interface: _
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, trusted_client_state_chain_id);

//         let counterparty_chain_id = trusted_client_state.chain_id();

//         assert_eq!(channel_proof_chain_id, this_chain_id);

//         effect(id(
//             counterparty_chain_id,
//             MsgChannelOpenTryData(MsgChannelOpenTry {
//                 port_id: channel.counterparty.port_id.clone(),
//                 channel: Channel {
//                     state: channel::state::State::Tryopen,
//                     ordering: channel.ordering,
//                     counterparty: channel::counterparty::Counterparty {
//                         port_id: event.port_id.clone(),
//                         channel_id: event.channel_id.clone().to_string(),
//                     },
//                     connection_hops:
// vec![connection.counterparty.connection_id.parse().unwrap()],
// version: event.version.clone(),                 },
//                 // NOTE: Review behaviour here
//                 counterparty_version: event.version,
//                 proof_init: channel_proof,
//                 proof_height: channel_proof_height.into(),
//             }),
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgChannelOpenAck>
// {
//     type AggregatedData = HList![
//         Identified<IbcState<ClientStatePath>>,
//         Identified<IbcProof<ChannelEndPath>>,
//         Identified<IbcState<ChannelEndPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateMsgChannelOpenAck {
//                     event_height: _,
//                     event,
//                 },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: trusted_client_state_chain_id,
//                 t: IbcState {
//                     path: ClientStatePath {
//                         client_id: _trusted_client_state_client_id
//                     },
//                     height: _trusted_client_state_fetched_at_height,
//                     state: trusted_client_state
//                 },
//             },
//             Identified {
//                 chain_id: channel_proof_chain_id,
//                 t: IbcProof {
//                     height: channel_proof_height,
//                     proof: channel_proof,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: channel_end_chain_id,
//                 t: IbcState {
//                     path: _,
//                     height: _,
//                     state: channel,
//                     ibc_interface: _
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         let counterparty_chain_id = trusted_client_state.chain_id();

//         assert_eq!(trusted_client_state_chain_id, this_chain_id);
//         assert_eq!(channel_proof_chain_id, this_chain_id);
//         assert_eq!(channel_end_chain_id, this_chain_id);

//         effect(id(
//             counterparty_chain_id,
//             MsgChannelOpenAckData(MsgChannelOpenAck {
//                 port_id: channel.counterparty.port_id.clone(),
//                 channel_id: event.counterparty_channel_id,
//                 counterparty_channel_id: event.channel_id,
//                 counterparty_version: event.version,
//                 proof_try: channel_proof,
//                 proof_height: channel_proof_height,
//             }),
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgChannelOpenConfirm>
// {
//     type AggregatedData = HList![
//         Identified<IbcState<ClientStatePath>>,
//         Identified<IbcProof<ChannelEndPath>>,
//         Identified<IbcState<ChannelEndPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateMsgChannelOpenConfirm {
//                     event_height: _,
//                     event,
//                 },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: trusted_client_state_chain_id,
//                 t: IbcState {
//                     path: ClientStatePath {
//                         client_id: _trusted_client_state_client_id
//                     },
//                     height: _trusted_client_state_fetched_at_height,
//                     state: trusted_client_state,
//                     ibc_interface: _
//                 },
//             },
//             Identified {
//                 chain_id: channel_proof_chain_id,
//                 t: IbcProof {
//                     height: channel_proof_height,
//                     proof: channel_proof,
//                     path: _,
//                 },
//             },
//             Identified {
//                 chain_id: channel_end_chain_id,
//                 t: IbcState {
//                     path: _,
//                     height: _,
//                     state: channel,
//                     ibc_interface: _
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, trusted_client_state_chain_id);
//         assert_eq!(this_chain_id, channel_proof_chain_id);
//         assert_eq!(channel_end_chain_id, this_chain_id);

//         let counterparty_chain_id = trusted_client_state.chain_id();

//         effect(id(
//             counterparty_chain_id,
//             MsgChannelOpenConfirmData(MsgChannelOpenConfirm {
//                 port_id: channel.counterparty.port_id,
//                 channel_id: event.counterparty_channel_id,
//                 proof_ack: channel_proof,
//                 proof_height: channel_proof_height.into(),
//             }),
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgRecvPacket>
// {
//     type AggregatedData = HList![
//         Identified<IbcState<ClientStatePath>>,
//         Identified<IbcProof<CommitmentPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateMsgRecvPacket {
//                     event_height: _,
//                     event,
//                 },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: trusted_client_state_chain_id,
//                 t: IbcState {
//                     path: ClientStatePath {
//                         client_id: _trusted_client_state_client_id
//                     },
//                     height: _trusted_client_state_fetched_at_height,
//                     state: trusted_client_state,
//                     ibc_interface: _
//                 },
//             },
//             Identified {
//                 chain_id: commitment_proof_chain_id,
//                 t: IbcProof {
//                     height: commitment_proof_height,
//                     proof: commitment_proof,
//                     path: _,
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, trusted_client_state_chain_id);
//         assert_eq!(this_chain_id, commitment_proof_chain_id);

//         debug!("aggregate recv_packet");

//         let counterparty_chain_id = trusted_client_state.chain_id();

//         effect(id(
//             counterparty_chain_id,
//             MsgRecvPacketData(MsgRecvPacket {
//                 packet: Packet {
//                     sequence: event.packet_sequence,
//                     source_port: event.packet_src_port,
//                     source_channel: event.packet_src_channel,
//                     destination_port: event.packet_dst_port,
//                     destination_channel: event.packet_dst_channel,
//                     data: event.packet_data_hex,
//                     timeout_height: event.packet_timeout_height,
//                     timeout_timestamp: event.packet_timeout_timestamp,
//                 },
//                 proof_commitment: commitment_proof,
//                 proof_height: commitment_proof_height,
//             }),
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgAckPacket>
// {
//     type AggregatedData = HList![
//         Identified<IbcState<ClientStatePath>>,
//         Identified<IbcProof<AcknowledgementPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateMsgAckPacket {
//                     event_height: _,
//                     event,
//                     tx_hash: _,
//                     counterparty_client_id: _,
//                 },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: trusted_client_state_chain_id,
//                 t: IbcState {
//                     path: ClientStatePath {
//                         client_id: _trusted_client_state_client_id
//                     },
//                     height: _trusted_client_state_fetched_at_height,
//                     state: trusted_client_state,
//                     ibc_interface: _
//                 },
//             },
//             Identified {
//                 chain_id: commitment_proof_chain_id,
//                 t: IbcProof {
//                     proof: acknowledgement_proof,
//                     height: acknowledgement_proof_height,
//                     path: _,
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, trusted_client_state_chain_id);
//         assert_eq!(commitment_proof_chain_id, this_chain_id);

//         debug!("aggregate ack_packet");

//         let counterparty_chain_id = trusted_client_state.chain_id();

//         effect(id(
//             counterparty_chain_id,
//             MsgAckPacketData(MsgAcknowledgement {
//                 proof_height: acknowledgement_proof_height,
//                 packet: Packet {
//                     sequence: event.packet_sequence,
//                     source_port: event.packet_src_port,
//                     source_channel: event.packet_src_channel,
//                     destination_port: event.packet_dst_port,
//                     destination_channel: event.packet_dst_channel,
//                     data: event.packet_data_hex,
//                     timeout_height: event.packet_timeout_height,
//                     timeout_timestamp: event.packet_timeout_timestamp,
//                 },
//                 acknowledgement: event.packet_ack_hex,
//                 proof_acked: acknowledgement_proof,
//             }),
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateMsgTimeout>
// {
//     type AggregatedData = HList![
//         Identified<IbcProof<ReceiptPath>>,
//         Identified<IbcState<ReceiptPath>>,
//     ];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregateMsgTimeout { packet },
//         }: Self,
//         hlist_pat![
//             Identified {
//                 chain_id: _,
//                 t: IbcProof {
//                     proof: proof_unreceived,
//                     height: proof_unreceived_height,
//                     // TODO: Assert these against the packet
//                     path: proof_unreceived_path,
//                 },
//             },
//             Identified {
//                 chain_id: _,
//                 t: IbcState {
//                     state: packet_receipt,
//                     height: packet_receipt_height,
//                     path: packet_receipt_path,
//                     ibc_interface: _
//                 },
//             },
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(proof_unreceived_path, packet_receipt_path);
//         assert_eq!(proof_unreceived_height, packet_receipt_height);

//         if packet_receipt {
//             info!(
//                 sequence = %packet.sequence,
//                 source_port = %packet.source_port,
//                 source_channel = %packet.source_channel,
//                 destination_port = %packet.destination_port,
//                 destination_channel = %packet.destination_channel,
//                 "packet received, cancelling timeout"
//             );

//             noop()
//         } else {
//             seq([
//                 // void(wait(id(
//                 //     this_chain_id.clone(),
//                 //     WaitForTrustedHeight {
//                 //         client_id,
//                 //         counterparty_client_id,
//                 //         counterparty_chain_id,
//                 //         height: proof_unreceived_height,
//                 //     },
//                 // ))),
//                 effect(id(
//                     this_chain_id,
//                     MsgTimeoutData(MsgTimeout {
//                         packet,
//                         proof_unreceived,
//                         proof_height: proof_unreceived_height,
//                         next_sequence_recv: proof_unreceived_path.sequence,
//                     }),
//                 )),
//             ])
//         }
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateWaitForPacketReceipt>
// {
//     type AggregatedData = HList![Identified<IbcState<ReceiptPath>>,];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregateWaitForPacketReceipt { packet },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: _,
//             t: IbcState {
//                 state: packet_receipt,
//                 height: _packet_receipt_height,
//                 path: packet_receipt_path,
//                 ibc_interface: _
//             },
//         },]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         if packet_receipt {
//             info!(
//                 sequence = %packet.sequence,
//                 source_port = %packet.source_port,
//                 source_channel = %packet.source_channel,
//                 destination_port = %packet.destination_port,
//                 destination_channel = %packet.destination_channel,
//                 "packet received"
//             );

//             noop()
//         } else {
//             seq([
//                 defer_relative(1),
//                 aggregate(
//                     [fetch(id(
//                         this_chain_id.clone(),
//                         FetchState {
//                             at: QueryHeight::Latest,
//                             path: packet_receipt_path.into(),
//                         },
//                     ))],
//                     [],
//                     id(
//                         this_chain_id.clone(),
//                         AggregateWaitForPacketReceipt { packet },
//                     ),
//                 ),
//             ])
//         }
//     }
// }
// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateFetchCounterpartyStateProof>
// {
//     type AggregatedData = HList![Identified<IbcState<ClientStatePath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateFetchCounterpartyStateProof {
//                     counterparty_client_id: _,
//                     fetch: fetch_,
//                 },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: trusted_client_state_chain_id,
//             t: IbcState {
//                 height: _,
//                 path: _,
//                 state: trusted_client_state,
//                 ibc_interface: _
//             },
//         }]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, trusted_client_state_chain_id);

//         let counterparty_chain_id = trusted_client_state.chain_id();

//         fetch(id(counterparty_chain_id, fetch_.into()))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateFetchReceiptPathProofFromChannelAndPort>
// {
//     type AggregatedData = HList![Identified<IbcState<NextSequenceRecvPath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t:
//                 AggregateFetchReceiptPathProofFromChannelAndPort {
//                     port_id,
//                     channel_id,
//                 },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: next_sequence_recv_chain_id,
//             t: IbcState {
//                 path: next_sequence_recv_path,
//                 height,
//                 state: next_sequence_recv,
//                 ibc_interface: _
//             },
//         },]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, next_sequence_recv_chain_id);

//         assert_eq!(next_sequence_recv_path.port_id, port_id);
//         assert_eq!(next_sequence_recv_path.channel_id, channel_id);

//         fetch(id(
//             this_chain_id,
//             FetchProof {
//                 at: height,
//                 path: ReceiptPath {
//                     port_id,
//                     channel_id,
//                     sequence: NonZeroU64::new(next_sequence_recv).unwrap(),
//                 }
//                 .into(),
//             },
//         ))
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateWaitForNextConnectionSequence>
// {
//     type AggregatedData =
// HList![Identified<IbcState<NextConnectionSequencePath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregateWaitForNextConnectionSequence { sequence },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: next_connection_sequence_chain_id,
//             t: IbcState {
//                 path: NextConnectionSequencePath {},
//                 height: _,
//                 state: next_connection_sequence,
//                 ibc_interface: _
//             },
//         },]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, next_connection_sequence_chain_id);

//         if next_connection_sequence >= sequence {
//             noop()
//         } else {
//             seq([
//                 defer_relative(1),
//                 aggregate(
//                     [fetch(id(
//                         this_chain_id.clone(),
//                         FetchState {
//                             at: QueryHeight::Latest,
//                             path: NextConnectionSequencePath {}.into(),
//                         },
//                     ))],
//                     [],
//                     id(
//                         this_chain_id,
//                         AggregateWaitForNextConnectionSequence { sequence },
//                     ),
//                 ),
//             ])
//         }
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateWaitForNextClientSequence>
// {
//     type AggregatedData =
// HList![Identified<IbcState<NextClientSequencePath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregateWaitForNextClientSequence { sequence },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: next_client_sequence_chain_id,
//             t: IbcState {
//                 path: NextClientSequencePath {},
//                 height: _,
//                 state: next_client_sequence,
//                 ibc_interface: _,
//             },
//         },]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, next_client_sequence_chain_id);

//         if next_client_sequence >= sequence {
//             noop()
//         } else {
//             seq([
//                 defer_relative(1),
//                 aggregate(
//                     [fetch(id(
//                         this_chain_id.clone(),
//                         FetchState {
//                             at: QueryHeight::Latest,
//                             path: NextClientSequencePath {}.into(),
//                         },
//                     ))],
//                     [],
//                     id(
//                         this_chain_id,
//                         AggregateWaitForNextClientSequence { sequence },
//                     ),
//                 ),
//             ])
//         }
//     }
// }

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for Identified<AggregateWaitForConnectionOpen>
// {
//     type AggregatedData = HList![Identified<IbcState<ConnectionPath>>];

//     fn aggregate(
//         Identified {
//             chain_id: this_chain_id,
//             t: AggregateWaitForConnectionOpen { connection_id },
//         }: Self,
//         hlist_pat![Identified {
//             chain_id: connection_state_client_id,
//             t: IbcState {
//                 path: ConnectionPath {
//                     connection_id: path_connection_id
//                 },
//                 height: _,
//                 state: connection,
//                 ibc_interface: _,
//             },
//         },]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//         assert_eq!(this_chain_id, connection_state_client_id);
//         assert_eq!(connection_id, path_connection_id);

//         if connection.state == connection::state::State::Open {
//             noop()
//         } else {
//             seq([
//                 defer_relative(1),
//                 aggregate(
//                     [fetch(id(
//                         this_chain_id.clone(),
//                         FetchState {
//                             at: QueryHeight::Latest,
//                             path: ConnectionPath {
//                                 connection_id: connection_id.clone(),
//                             }
//                             .into(),
//                         },
//                     ))],
//                     [],
//                     id(
//                         this_chain_id,
//                         AggregateWaitForConnectionOpen { connection_id },
//                     ),
//                 ),
//             ])
//         }
//     }
// }

#[queue_msg]
pub struct AggregateFetchCounterpartyChannelAndConnection {
    pub counterparty_port_id: PortId,
    pub counterparty_channel_id: ChannelId,
}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateFetchCounterpartyChannelAndConnection
{
    type AggregatedData = HList![DecodedClientStateMeta];

    fn aggregate(
        Self {
            counterparty_port_id,
            counterparty_channel_id,
        }: Self,
        hlist_pat![client_state]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        conc([
            fetch(FetchState {
                chain_id: client_state.state.chain_id.clone(),
                at: QueryHeight::Latest,
                path: ChannelEndPath {
                    port_id: counterparty_port_id.clone(),
                    channel_id: counterparty_channel_id.clone(),
                }
                .into(),
            }),
            fetch_connection_from_channel_info(
                client_state.state.chain_id,
                QueryHeight::Latest,
                counterparty_port_id,
                counterparty_channel_id,
            ),
        ])
    }
}

// impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
//     for MakeMsgConnectionOpenTry
// {
//     type AggregatedData = HList![
//         // DecodedClientStateMeta,
//         IbcState<ClientStatePath>,
//         IbcProof<ClientStatePath>,
//         IbcProof<ClientConsensusStatePath>,
//         IbcState<ConnectionPath>,
//         IbcProof<ConnectionPath>,
//     ];

//     fn aggregate(
//         Self { event }: Self,
//         hlist_pat![
//             // client_meta,
//             client_state,
//             client_state_proof,
//             consensus_state_proof,
//             connection_state,
//             connection_proof
//         ]: Self::AggregatedData,
//     ) -> Op<VoyagerMessage<D, F, A>> {
//     }
// }

/// Returns the same data as [`AggregateFetchCounterpartyChannelAndConnection`],
/// except this also requires the source channel in the input data. Useful when
/// only the source channel ids are known.
#[queue_msg]
pub struct AggregateFetchCounterpartyChannelAndConnectionFromSourceChannel {}

impl<D: Member, F: Member, A: Member> UseAggregate<VoyagerMessage<D, F, A>>
    for AggregateFetchCounterpartyChannelAndConnectionFromSourceChannel
{
    type AggregatedData = HList![DecodedClientStateMeta, IbcState<ChannelEndPath>];

    fn aggregate(
        Self {}: Self,
        hlist_pat![client_state, channel_state]: Self::AggregatedData,
    ) -> Op<VoyagerMessage<D, F, A>> {
        conc([
            fetch(FetchState {
                chain_id: client_state.state.chain_id.clone(),
                at: QueryHeight::Latest,
                path: ChannelEndPath {
                    port_id: channel_state.state.counterparty.port_id.clone(),
                    channel_id: channel_state.state.counterparty.channel_id.parse().unwrap(),
                }
                .into(),
            }),
            fetch_connection_from_channel_info(
                client_state.state.chain_id,
                QueryHeight::Latest,
                channel_state.state.counterparty.port_id.clone(),
                channel_state.state.counterparty.channel_id.parse().unwrap(),
            ),
        ])
    }
}

/// Build the source and destination [`ChannelMetadata`] from the passed in
/// `data` and the source/destination channel ids of the packet event. This will
/// return the `(src, dst)` tuple in the correct order correlating to the passed
/// in src/dst ids.
pub fn mk_packet_metadata<D: Member>(
    data: VecDeque<Data<D>>,
    source_chain_id: String,
    destination_chain_id: String,
    packet_src_port: PortId,
    packet_src_channel: ChannelId,
    packet_dst_port: PortId,
    packet_dst_channel: ChannelId,
) -> (ChannelMetadata, ChannelMetadata) {
    mk_packet_metadata_from_hlist::<D>(
        <HList![
            IbcState<ConnectionPath>,
            IbcState<ConnectionPath>,
            IbcState<ChannelEndPath>,
            IbcState<ChannelEndPath>,
        ]>::try_from_iter(data)
        .unwrap(),
        source_chain_id,
        destination_chain_id,
        packet_src_port,
        packet_src_channel,
        packet_dst_port,
        packet_dst_channel,
    )
}

#[instrument(level = "debug")]
pub fn mk_packet_metadata_from_hlist<D: Member>(
    hlist_pat![connection_a, connection_b, channel_a, channel_b]: HList![
        IbcState<ConnectionPath>,
        IbcState<ConnectionPath>,
        IbcState<ChannelEndPath>,
        IbcState<ChannelEndPath>,
    ],
    source_chain_id: String,
    destination_chain_id: String,
    packet_src_port: PortId,
    packet_src_channel: ChannelId,
    packet_dst_port: PortId,
    packet_dst_channel: ChannelId,
) -> (ChannelMetadata, ChannelMetadata) {
    // a is source, b is destination
    let (source_channel, destination_channel) = if channel_a.chain_id == source_chain_id {
        (channel_a.state, channel_b.state)
    }
    // b is source, a is destination
    else if channel_b.chain_id == source_chain_id {
        (channel_b.state, channel_a.state)
    } else {
        dbg!(
            &connection_a,
            &connection_b,
            &channel_a,
            &channel_b,
            &source_chain_id,
            &destination_chain_id
        );
        panic!("invalid channels")
    };

    // a is source, b is destination
    let (source_connection, destination_connection) = if connection_a.chain_id == source_chain_id {
        (connection_a, connection_b)
    }
    // b is source, a is destination
    else if connection_b.chain_id == source_chain_id {
        (connection_b, connection_a)
    } else {
        error!("invalid connections");
        panic!("invalid connections, see logs for more details");
    };

    (
        ChannelMetadata {
            port_id: packet_src_port,
            channel_id: packet_src_channel,
            ordering: source_channel.ordering,
            version: source_channel.version,
            connection: ConnectionMetadata {
                client_id: source_connection.state.client_id,
                connection_id: source_connection.path.connection_id,
            },
        },
        ChannelMetadata {
            port_id: packet_dst_port,
            channel_id: packet_dst_channel,
            ordering: destination_channel.ordering,
            version: destination_channel.version,
            connection: ConnectionMetadata {
                client_id: destination_connection.state.client_id,
                connection_id: destination_connection.path.connection_id,
            },
        },
    )
}
