use std::{collections::VecDeque, fmt::Display, marker::PhantomData};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::H256,
    events::{
        ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck, ConnectionOpenInit,
        ConnectionOpenTry, RecvPacket, SendPacket,
    },
    ibc::core::{
        channel::{
            self, channel::Channel, msg_acknowledgement::MsgAcknowledgement,
            msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_try::MsgChannelOpenTry, msg_recv_packet::MsgRecvPacket,
            packet::Packet,
        },
        client::{height::IsHeight, msg_create_client::MsgCreateClient},
        commitment::merkle_prefix::MerklePrefix,
        connection::{
            self, msg_connection_open_ack::MsgConnectionOpenAck,
            msg_connection_open_confirm::MsgConnectionOpenConfirm,
            msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    proof::{
        self, AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath,
    },
    traits::{ChainIdOf, ChainOf, ClientState, HeightOf, LightClientBase},
    QueryHeight, DELAY_PERIOD,
};

use crate::{
    any_enum,
    data::{
        AcknowledgementProof, AnyData, ChannelEnd, ChannelEndProof, ClientConsensusStateProof,
        ClientStateProof, CommitmentProof, ConnectionEnd, ConnectionProof, Data,
        PacketAcknowledgement, SelfClientState, SelfConsensusState, TrustedClientState,
    },
    fetch,
    fetch::{
        AnyFetch, Fetch, FetchChannelEnd, FetchConnectionEnd, FetchPacketAcknowledgement,
        FetchStateProof, FetchTrustedClientState, FetchUpdateHeaders,
    },
    identified, msg,
    msg::{
        AnyMsg, Msg, MsgAckPacketData, MsgChannelOpenAckData, MsgChannelOpenConfirmData,
        MsgChannelOpenTryData, MsgConnectionOpenAckData, MsgConnectionOpenConfirmData,
        MsgConnectionOpenTryData, MsgCreateClientData, MsgRecvPacketData,
    },
    use_aggregate::{do_aggregate, IsAggregateData, UseAggregate},
    wait,
    wait::{AnyWait, Wait, WaitForTrustedHeight},
    AnyLightClientIdentified, DoAggregate, Identified, LightClient, RelayerMsg,
};

any_enum! {
    /// Aggregate data, using data from [`AggregateData`]
    #[any = AnyAggregate]
    pub enum Aggregate<L: LightClient> {
        ConnectionOpenTry(AggregateConnectionOpenTry<L>),
        ConnectionOpenAck(AggregateConnectionOpenAck<L>),
        ConnectionOpenConfirm(AggregateConnectionOpenConfirm<L>),

        ChannelOpenTry(AggregateChannelOpenTry<L>),
        ChannelOpenAck(AggregateChannelOpenAck<L>),
        ChannelOpenConfirm(AggregateChannelOpenConfirm<L>),

        RecvPacket(AggregateRecvPacket<L>),
        AckPacket(AggregateAckPacket<L>),

        ConnectionFetchFromChannelEnd(AggregateConnectionFetchFromChannelEnd<L>),

        // Aggregate that fetches the connection info from the channel
        ChannelHandshakeUpdateClient(AggregateChannelHandshakeUpdateClient<L>),

        PacketUpdateClient(AggregatePacketUpdateClient<L>),

        WaitForTrustedHeight(AggregateWaitForTrustedHeight<L>),

        FetchCounterpartyStateproof(AggregateFetchCounterpartyStateProof<L>),

        UpdateClientFromClientId(AggregateUpdateClientFromClientId<L>),

        UpdateClient(AggregateUpdateClient<L>),
        UpdateClientWithCounterpartyChainIdData(AggregateUpdateClientWithCounterpartyChainId<L>),

        CreateClient(AggregateCreateClient<L>),

        AggregateMsgAfterUpdate(AggregateMsgAfterUpdate<L>),

        LightClientSpecific(LightClientSpecificAggregate<L>),
    }
}

impl<L: LightClient> identified!(Aggregate<L>) {
    pub fn handle(self, data: VecDeque<AnyLightClientIdentified<AnyData>>) -> Vec<RelayerMsg>
    where
        identified!(TrustedClientState<L>): IsAggregateData,
        identified!(TrustedClientState<L::Counterparty>): IsAggregateData,

        identified!(ClientStateProof<L>): IsAggregateData,
        identified!(ClientConsensusStateProof<L>): IsAggregateData,
        identified!(ConnectionProof<L>): IsAggregateData,
        identified!(ChannelEndProof<L>): IsAggregateData,
        identified!(CommitmentProof<L>): IsAggregateData,
        identified!(AcknowledgementProof<L>): IsAggregateData,

        identified!(SelfClientState<L::Counterparty>): IsAggregateData,
        identified!(SelfConsensusState<L::Counterparty>): IsAggregateData,

        identified!(ChannelEnd<L>): IsAggregateData,
        identified!(ConnectionEnd<L>): IsAggregateData,
        identified!(PacketAcknowledgement<L>): IsAggregateData,

        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,

        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L>)>,
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L::Counterparty>)>,

        AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L>)>,
        AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,

        AnyLightClientIdentified<AnyData>: From<identified!(Data<L>)>,
        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
    {
        let chain_id = self.chain_id;

        match self.data {
            Aggregate::ConnectionOpenTry(init) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: init,
                },
                data,
            )]
            .into(),
            Aggregate::ConnectionOpenAck(ack) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: ack,
                },
                data,
            )]
            .into(),
            Aggregate::ConnectionOpenConfirm(confirm) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: confirm,
                },
                data,
            )]
            .into(),
            Aggregate::ChannelOpenTry(try_) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: try_,
                },
                data,
            )]
            .into(),
            Aggregate::ChannelOpenAck(ack) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: ack,
                },
                data,
            )]
            .into(),
            Aggregate::ChannelOpenConfirm(confirm) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: confirm,
                },
                data,
            )]
            .into(),
            Aggregate::UpdateClientFromClientId(update_client) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: update_client,
                },
                data,
            )]
            .into(),
            Aggregate::UpdateClient(update_client) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: update_client,
                },
                data,
            )]
            .into(),
            Aggregate::UpdateClientWithCounterpartyChainIdData(aggregate) => {
                [do_aggregate::<L, _>(
                    Identified {
                        chain_id,
                        data: aggregate,
                    },
                    data,
                )]
                .into()
            }
            Aggregate::CreateClient(create_client) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: create_client,
                },
                data,
            )]
            .into(),
            Aggregate::AggregateMsgAfterUpdate(aggregate) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: aggregate,
                },
                data,
            )]
            .into(),
            Aggregate::LightClientSpecific(LightClientSpecificAggregate(aggregate)) => {
                L::Aggregate::do_aggregate(
                    Identified {
                        chain_id,
                        data: aggregate,
                    },
                    data,
                )
            }
            Aggregate::ConnectionFetchFromChannelEnd(aggregate) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: aggregate,
                },
                data,
            )]
            .into(),
            Aggregate::ChannelHandshakeUpdateClient(channel_handshake_update_client) => {
                [do_aggregate::<L, _>(
                    Identified {
                        chain_id,
                        data: channel_handshake_update_client,
                    },
                    data,
                )]
                .into()
            }
            Aggregate::PacketUpdateClient(packet_update_client) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: packet_update_client,
                },
                data,
            )]
            .into(),
            Aggregate::RecvPacket(recv_packet) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: recv_packet,
                },
                data,
            )]
            .into(),
            Aggregate::AckPacket(ack_packet) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: ack_packet,
                },
                data,
            )]
            .into(),
            Aggregate::WaitForTrustedHeight(agg) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: agg,
                },
                data,
            )]
            .into(),
            Aggregate::FetchCounterpartyStateproof(agg) => [do_aggregate::<L, _>(
                Identified {
                    chain_id,
                    data: agg,
                },
                data,
            )]
            .into(),
        }
    }
}

impl<L: LightClient> Display for Aggregate<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aggregate::ConnectionOpenTry(_) => write!(f, "ConnectionOpenTry"),
            Aggregate::ConnectionOpenAck(_) => write!(f, "ConnectionOpenAck"),
            Aggregate::ConnectionOpenConfirm(_) => write!(f, "ConnectionOpenConfirm"),
            Aggregate::ChannelOpenTry(_) => write!(f, "ChannelOpenTry"),
            Aggregate::ChannelOpenAck(_) => write!(f, "ChannelOpenAck"),
            Aggregate::ChannelOpenConfirm(_) => write!(f, "ChannelOpenConfirm"),
            Aggregate::RecvPacket(_) => write!(f, "RecvPacket"),
            Aggregate::AckPacket(_) => write!(f, "AckPacket"),
            Aggregate::ConnectionFetchFromChannelEnd(_) => {
                write!(f, "ConnectionFetchFromChannelEnd")
            }
            Aggregate::ChannelHandshakeUpdateClient(_) => {
                write!(f, "ChannelHandshakeUpdateClient")
            }
            Aggregate::PacketUpdateClient(msg) => {
                write!(
                    f,
                    "PacketUpdateClient::{}",
                    match msg.packet_event {
                        PacketEvent::Send(_) => "Send",
                        PacketEvent::Recv(_) => "Recv",
                    }
                )
            }
            Aggregate::WaitForTrustedHeight(_) => write!(f, "WaitForTrustedHeight"),
            Aggregate::FetchCounterpartyStateproof(_) => {
                write!(f, "FetchCounterpartyStateproof")
            }
            Aggregate::UpdateClientFromClientId(_) => write!(f, "UpdateClientFromClientId"),
            Aggregate::UpdateClient(_) => write!(f, "UpdateClient"),
            Aggregate::UpdateClientWithCounterpartyChainIdData(_) => {
                write!(f, "UpdateClientWithCounterpartyChainIdData")
            }
            Aggregate::CreateClient(_) => write!(f, "CreateClient"),
            Aggregate::AggregateMsgAfterUpdate(msg) => {
                write!(f, "AggregateMsgAfterUpdate::")?;
                match msg {
                    AggregateMsgAfterUpdate::ConnectionOpenTry(_) => {
                        write!(f, "ConnectionOpenTry")
                    }
                    AggregateMsgAfterUpdate::ConnectionOpenAck(_) => {
                        write!(f, "ConnectionOpenAck")
                    }
                    AggregateMsgAfterUpdate::ConnectionOpenConfirm(_) => {
                        write!(f, "ConnectionOpenConfirm")
                    }
                    AggregateMsgAfterUpdate::ChannelOpenTry(_) => write!(f, "ChannelOpenTry"),
                    AggregateMsgAfterUpdate::ChannelOpenAck(_) => write!(f, "ChannelOpenAck"),
                    AggregateMsgAfterUpdate::ChannelOpenConfirm(_) => {
                        write!(f, "ChannelOpenConfirm")
                    }
                    AggregateMsgAfterUpdate::RecvPacket(_) => write!(f, "RecvPacket"),
                    AggregateMsgAfterUpdate::AckPacket(_) => write!(f, "AckPacket"),
                }
            }
            Aggregate::LightClientSpecific(agg) => write!(f, "LightClientSpecific({})", agg.0),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionOpenTry<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ConnectionOpenInit<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionOpenAck<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ConnectionOpenTry<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionOpenConfirm<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ConnectionOpenAck<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenTry<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ChannelOpenInit,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenAck<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ChannelOpenTry,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenConfirm<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ChannelOpenAck,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateRecvPacket<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: SendPacket,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateAckPacket<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: RecvPacket,
    // HACK: Need to pass the block hash through, figure out a better/cleaner way to do this
    pub block_hash: H256,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionFetchFromChannelEnd<L: LightClient> {
    pub at: HeightOf<ChainOf<L>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelHandshakeUpdateClient<L: LightClient> {
    // Will be threaded through to the update msg
    pub update_to: HeightOf<L::HostChain>,
    pub event_height: HeightOf<L::HostChain>,
    pub channel_handshake_event: ChannelHandshakeEvent,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum ChannelHandshakeEvent {
    Init(ChannelOpenInit),
    Try(ChannelOpenTry),
    Ack(ChannelOpenAck),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregatePacketUpdateClient<L: LightClient> {
    // Will be threaded through to the update msg
    pub update_to: HeightOf<L::HostChain>,
    pub event_height: HeightOf<L::HostChain>,
    pub block_hash: H256,
    pub packet_event: PacketEvent,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum PacketEvent {
    Send(SendPacket),
    Recv(RecvPacket),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateFetchCounterpartyStateProof<L: LightClient> {
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub fetch: FetchStateProof<L::Counterparty>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClientFromClientId<L: LightClient> {
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClient<L: LightClient> {
    pub update_to: HeightOf<L::HostChain>,
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateWaitForTrustedHeight<L: LightClient> {
    pub wait_for: HeightOf<L::HostChain>,
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClientWithCounterpartyChainId<L: LightClient> {
    pub update_to: HeightOf<L::HostChain>,
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateMsgUpdateClient<L: LightClient> {
    pub update_to: HeightOf<L::HostChain>,
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateCreateClient<L: LightClient> {
    pub config: L::Config,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientSpecificAggregate<L: LightClient>(pub L::Aggregate);

/// Messages that will be re-queued after an update.
#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum AggregateMsgAfterUpdate<L: LightClient> {
    ConnectionOpenTry(AggregateConnectionOpenTry<L>),
    ConnectionOpenAck(AggregateConnectionOpenAck<L>),
    ConnectionOpenConfirm(AggregateConnectionOpenConfirm<L>),

    ChannelOpenTry(AggregateChannelOpenTry<L>),
    ChannelOpenAck(AggregateChannelOpenAck<L>),
    ChannelOpenConfirm(AggregateChannelOpenConfirm<L>),

    RecvPacket(AggregateRecvPacket<L>),
    AckPacket(AggregateAckPacket<L>),
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateChannelHandshakeUpdateClient<L>)
where
    identified!(ConnectionEnd<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    type AggregatedData = HList![identified!(ConnectionEnd<L>)];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateChannelHandshakeUpdateClient {
                    update_to,
                    channel_handshake_event,
                    event_height,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: ConnectionEnd(connection),
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);

        let event_msg = match channel_handshake_event {
            ChannelHandshakeEvent::Init(init) => {
                AggregateMsgAfterUpdate::ChannelOpenTry(AggregateChannelOpenTry {
                    event_height,
                    event: init,
                })
            }
            ChannelHandshakeEvent::Try(try_) => {
                AggregateMsgAfterUpdate::ChannelOpenAck(AggregateChannelOpenAck {
                    event_height,
                    event: try_,
                })
            }
            ChannelHandshakeEvent::Ack(ack) => {
                AggregateMsgAfterUpdate::ChannelOpenConfirm(AggregateChannelOpenConfirm {
                    event_height,
                    event: ack,
                })
            }
        };

        RelayerMsg::Aggregate {
            data: [].into(),
            queue: [mk_aggregate_wait_for_update(
                this_chain_id.clone(),
                connection.client_id,
                connection.counterparty.client_id,
                update_to,
            )]
            .into(),
            receiver: AnyLightClientIdentified::from(Identified::new(
                this_chain_id,
                Aggregate::AggregateMsgAfterUpdate(event_msg),
            )),
        }
    }
}

pub fn mk_aggregate_wait_for_update<L: LightClient>(
    chain_id: ChainIdOf<ChainOf<L>>,
    client_id: L::ClientId,
    counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    wait_for: HeightOf<L::HostChain>,
) -> RelayerMsg
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    RelayerMsg::Aggregate {
        queue: [fetch::<L>(
            chain_id.clone(),
            FetchTrustedClientState {
                at: QueryHeight::Latest,
                client_id: client_id.clone().clone(),
            },
        )]
        .into(),
        data: [].into(),
        receiver: AnyLightClientIdentified::from(Identified::new(
            chain_id,
            Aggregate::<L>::WaitForTrustedHeight(AggregateWaitForTrustedHeight {
                wait_for,
                client_id,
                counterparty_client_id,
            }),
        )),
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregatePacketUpdateClient<L>)
where
    identified!(ConnectionEnd<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    type AggregatedData = HList![identified!(ConnectionEnd<L>)];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregatePacketUpdateClient {
                    update_to,
                    event_height,
                    block_hash,
                    packet_event,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: ConnectionEnd(connection),
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);

        let event = match packet_event {
            PacketEvent::Send(send) => Aggregate::AggregateMsgAfterUpdate(
                AggregateMsgAfterUpdate::RecvPacket(AggregateRecvPacket {
                    event_height,
                    event: send,
                }),
            ),
            PacketEvent::Recv(recv) => Aggregate::AggregateMsgAfterUpdate(
                AggregateMsgAfterUpdate::AckPacket(AggregateAckPacket {
                    event_height,
                    event: recv,
                    block_hash,
                    counterparty_client_id: connection.counterparty.client_id.clone(),
                }),
            ),
        };

        let agg = RelayerMsg::Aggregate {
            queue: [fetch::<L>(
                this_chain_id.clone().clone(),
                FetchTrustedClientState {
                    at: QueryHeight::Latest,
                    client_id: connection.client_id.clone().clone(),
                },
            )]
            .into(),
            data: [].into(),
            receiver: AnyLightClientIdentified::from(Identified::new(
                this_chain_id.clone(),
                Aggregate::<L>::WaitForTrustedHeight(AggregateWaitForTrustedHeight {
                    wait_for: update_to,
                    client_id: connection.client_id.clone().clone(),
                    counterparty_client_id: connection.counterparty.client_id.clone(),
                }),
            )),
        };

        RelayerMsg::Aggregate {
            data: [].into(),
            queue: [agg].into(),
            receiver: AnyLightClientIdentified::from(Identified::new(this_chain_id, event)),
        }
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateConnectionFetchFromChannelEnd<L>)
where
    identified!(ChannelEnd<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
{
    type AggregatedData = HList![identified!(ChannelEnd<L>)];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data: AggregateConnectionFetchFromChannelEnd { at },
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: ChannelEnd {
                channel,
                __marker: _
            },
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);

        fetch(
            this_chain_id,
            FetchConnectionEnd {
                at,
                connection_id: channel.connection_hops[0].clone(),
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateUpdateClientFromClientId<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    type AggregatedData = HList![identified!(TrustedClientState<L>)];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateUpdateClientFromClientId {
                    client_id,
                    counterparty_client_id,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: TrustedClientState {
                fetched_at,
                client_id: trusted_client_state_client_id,
                trusted_client_state,
            },
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);
        assert_eq!(trusted_client_state_client_id, client_id);

        let counterparty_chain_id = trusted_client_state.chain_id();

        RelayerMsg::Aggregate {
            queue: [fetch::<L::Counterparty>(
                counterparty_chain_id.clone(),
                FetchTrustedClientState {
                    // NOTE: Use latest here since on client creation, the counterparty may not yet exist at the latest height.
                    //
                    // - client created on chain A of chain B at height 1-10, trusting height 2-10 of B
                    // - client created on chain B of chain A at height 2-15, trusting height 1-15 of A
                    // - attempt to update chain A by updating from it's latest trusted height, which is 2-10 - but the client state of B on doesn't exist at height 2-10
                    //
                    // Note that updating chain B would work in this situation, since it was created after it's counterparty.
                    //
                    // Since this query is only to fetch the chain id of the counterparty, all that matters is that we get *a* client state, since it's expected that the chain id in a client state will never change.
                    //
                    // REVIEW: How will this work with chain upgrades? Since the revision number will change (on cosmos chains), so will the chain id - this will need to be handled
                    // at: QueryHeight::Specific(trusted_client_state.height()),
                    at: QueryHeight::Latest,
                    client_id: counterparty_client_id.clone(),
                },
            )]
            .into(),
            data: [].into(),
            receiver: AnyLightClientIdentified::from(Identified::new(
                this_chain_id,
                Aggregate::UpdateClientWithCounterpartyChainIdData(
                    AggregateUpdateClientWithCounterpartyChainId {
                        update_to: fetched_at,
                        client_id,
                        counterparty_client_id,
                        counterparty_chain_id,
                    },
                ),
            )),
        }
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateUpdateClient<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    type AggregatedData = HList![identified!(TrustedClientState<L>)];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateUpdateClient {
                    update_to,
                    client_id: update_client_id,
                    counterparty_client_id: update_counterparty_client_id,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: TrustedClientState {
                fetched_at: _,
                client_id: trusted_client_state_client_id,
                trusted_client_state,
            },
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);
        assert_eq!(update_client_id, trusted_client_state_client_id);

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        RelayerMsg::Aggregate {
            queue: [fetch::<L::Counterparty>(
                counterparty_chain_id.clone(),
                FetchTrustedClientState {
                    at: QueryHeight::Latest,
                    client_id: update_counterparty_client_id.clone(),
                },
            )]
            .into(),
            data: [].into(),
            receiver: AnyLightClientIdentified::from(Identified::new(
                this_chain_id,
                Aggregate::UpdateClientWithCounterpartyChainIdData(
                    AggregateUpdateClientWithCounterpartyChainId {
                        update_to,
                        client_id: update_client_id,
                        counterparty_client_id: update_counterparty_client_id,
                        counterparty_chain_id,
                    },
                ),
            )),
        }
    }
}

impl<L: LightClient> UseAggregate<L>
    for identified!(AggregateUpdateClientWithCounterpartyChainId<L>)
where
    identified!(TrustedClientState<L::Counterparty>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    type AggregatedData = HList![identified!(TrustedClientState<L::Counterparty>)];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateUpdateClientWithCounterpartyChainId {
                    update_to,
                    client_id: update_client_id,
                    counterparty_client_id: update_counterparty_client_id,
                    counterparty_chain_id: update_counterparty_chain_id,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: counterparty_chain_id,
            data: TrustedClientState {
                fetched_at: _,
                client_id: latest_trusted_client_state_client_id,
                trusted_client_state
            },
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        let self_chain_id: ChainIdOf<ChainOf<L>> = trusted_client_state.chain_id();

        assert_eq!(this_chain_id, self_chain_id);
        assert_eq!(
            latest_trusted_client_state_client_id,
            update_counterparty_client_id
        );
        assert_eq!(counterparty_chain_id, update_counterparty_chain_id);

        fetch::<L>(
            this_chain_id,
            FetchUpdateHeaders {
                client_id: update_client_id,
                counterparty_client_id: update_counterparty_client_id,
                counterparty_chain_id,
                update_from: trusted_client_state.height(),
                update_to,
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateWaitForTrustedHeight<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L::Counterparty>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    type AggregatedData = HList![identified!(TrustedClientState<L>)];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateWaitForTrustedHeight {
                    wait_for,
                    client_id,
                    counterparty_client_id,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: trusted_client_state_chain_id,
            data: TrustedClientState {
                fetched_at: _,
                client_id: trusted_client_state_client_id,
                trusted_client_state
            },
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(trusted_client_state_client_id, client_id);
        assert_eq!(trusted_client_state_chain_id, this_chain_id);

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        tracing::debug!("building WaitForTrustedHeight");

        wait::<L::Counterparty>(
            counterparty_chain_id,
            WaitForTrustedHeight {
                height: wait_for,
                client_id: counterparty_client_id,
                counterparty_client_id: client_id,
                counterparty_chain_id: this_chain_id,
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateMsgAfterUpdate<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<L>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
{
    type AggregatedData = HList![identified!(TrustedClientState<L>)];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data: msg_to_aggregate,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: TrustedClientState {
                fetched_at: trusted_client_state_fetched_at_height,
                client_id: trusted_client_state_client_id,
                trusted_client_state
            },
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);
        // assert_eq!(client_id, trusted_client_state_client_id);

        match msg_to_aggregate {
            AggregateMsgAfterUpdate::ConnectionOpenTry(AggregateConnectionOpenTry {
                event_height,
                event,
            }) => {
                let consensus_state_height = trusted_client_state_fetched_at_height;

                assert_eq!(
                    consensus_state_height.revision_number(),
                    event_height.revision_number(),
                    "{consensus_state_height}, {event_height}",
                );

                assert!(
                    consensus_state_height.revision_height() >= event_height.revision_height(),
                    "{} < {}",
                    consensus_state_height.revision_height(),
                    event_height.revision_height()
                );

                let trusted_client_state_height = trusted_client_state.height();

                RelayerMsg::Aggregate {
                    data: [AnyLightClientIdentified::from(Identified::new(
                        this_chain_id.clone(),
                        Data::TrustedClientState(TrustedClientState {
                            fetched_at: trusted_client_state_fetched_at_height,
                            client_id: trusted_client_state_client_id,
                            trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ClientStatePath(ClientStatePath {
                                    client_id: event.client_id.clone().into(),
                                }),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ClientConsensusStatePath(
                                    ClientConsensusStatePath {
                                        client_id: event.client_id.clone().into(),
                                        height: trusted_client_state_height,
                                    },
                                ),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ConnectionPath(ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchConnectionEnd {
                                at: trusted_client_state_fetched_at_height,
                                connection_id: event.connection_id.clone(),
                            },
                        ),
                    ]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ConnectionOpenTry(AggregateConnectionOpenTry {
                            event_height,
                            event,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::ConnectionOpenAck(AggregateConnectionOpenAck {
                event_height,
                event,
            }) => {
                let consensus_state_height = trusted_client_state_fetched_at_height;

                assert_eq!(
                    consensus_state_height.revision_number(),
                    event_height.revision_number(),
                    "{consensus_state_height}, {event_height}",
                );

                assert!(
                    consensus_state_height.revision_height() >= event_height.revision_height(),
                    "{} < {}",
                    consensus_state_height.revision_height(),
                    event_height.revision_height()
                );

                let trusted_client_state_height = trusted_client_state.height();

                RelayerMsg::Aggregate {
                    data: [AnyLightClientIdentified::from(Identified::new(
                        this_chain_id.clone(),
                        Data::TrustedClientState(TrustedClientState {
                            fetched_at: trusted_client_state_fetched_at_height,
                            client_id: trusted_client_state_client_id,
                            trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ClientStatePath(ClientStatePath {
                                    client_id: event.client_id.clone().into(),
                                }),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ClientConsensusStatePath(
                                    ClientConsensusStatePath {
                                        client_id: event.client_id.clone().into(),
                                        height: trusted_client_state_height,
                                    },
                                ),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ConnectionPath(ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchConnectionEnd {
                                at: trusted_client_state_fetched_at_height,
                                connection_id: event.connection_id.clone(),
                            },
                        ),
                    ]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ConnectionOpenAck(AggregateConnectionOpenAck {
                            event_height,
                            event,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::ConnectionOpenConfirm(AggregateConnectionOpenConfirm {
                event_height,
                event,
            }) => {
                let consensus_state_height = trusted_client_state_fetched_at_height;

                assert_eq!(
                    consensus_state_height.revision_number(),
                    event_height.revision_number(),
                    "{consensus_state_height}, {event_height}",
                );

                assert!(
                    consensus_state_height.revision_height() >= event_height.revision_height(),
                    "{} < {}",
                    consensus_state_height.revision_height(),
                    event_height.revision_height()
                );

                RelayerMsg::Aggregate {
                    data: [AnyLightClientIdentified::from(Identified::new(
                        this_chain_id.clone(),
                        Data::TrustedClientState(TrustedClientState {
                            fetched_at: trusted_client_state_fetched_at_height,
                            client_id: trusted_client_state_client_id,
                            trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [fetch::<L>(
                        this_chain_id.clone(),
                        FetchStateProof {
                            at: trusted_client_state_fetched_at_height,
                            path: proof::Path::ConnectionPath(ConnectionPath {
                                connection_id: event.connection_id.clone(),
                            }),
                        },
                    )]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ConnectionOpenConfirm(AggregateConnectionOpenConfirm {
                            event_height,
                            event,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::ChannelOpenTry(AggregateChannelOpenTry {
                event_height,
                event,
            }) => {
                let consensus_state_height = trusted_client_state_fetched_at_height;

                assert_eq!(
                    consensus_state_height.revision_number(),
                    event_height.revision_number(),
                    "{consensus_state_height}, {event_height}",
                );

                assert!(
                    consensus_state_height.revision_height() >= event_height.revision_height(),
                    "{} < {}",
                    consensus_state_height.revision_height(),
                    event_height.revision_height()
                );

                RelayerMsg::Aggregate {
                    data: [AnyLightClientIdentified::from(Identified::new(
                        this_chain_id.clone(),
                        Data::TrustedClientState(TrustedClientState {
                            fetched_at: trusted_client_state_fetched_at_height,
                            client_id: trusted_client_state_client_id,
                            trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        RelayerMsg::Aggregate {
                            data: [].into(),
                            queue: [fetch::<L>(
                                this_chain_id.clone(),
                                FetchChannelEnd {
                                    at: trusted_client_state_fetched_at_height,
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                },
                            )]
                            .into(),
                            receiver: AnyLightClientIdentified::from(Identified::new(
                                this_chain_id.clone(),
                                Aggregate::ConnectionFetchFromChannelEnd(
                                    AggregateConnectionFetchFromChannelEnd {
                                        at: trusted_client_state_fetched_at_height,
                                    },
                                ),
                            )),
                        },
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ChannelEndPath(ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchChannelEnd {
                                at: trusted_client_state_fetched_at_height,
                                port_id: event.port_id.clone(),
                                channel_id: event.channel_id.clone(),
                            },
                        ),
                    ]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ChannelOpenTry(AggregateChannelOpenTry {
                            event_height,
                            event,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::ChannelOpenAck(AggregateChannelOpenAck {
                event_height,
                event,
            }) => {
                let consensus_state_height = trusted_client_state_fetched_at_height;

                assert_eq!(
                    consensus_state_height.revision_number(),
                    event_height.revision_number(),
                    "{consensus_state_height}, {event_height}",
                );

                assert!(
                    consensus_state_height.revision_height() >= event_height.revision_height(),
                    "{} < {}",
                    consensus_state_height.revision_height(),
                    event_height.revision_height()
                );

                // RelayerMsg::Sequence([].into());
                RelayerMsg::Aggregate {
                    data: [AnyLightClientIdentified::from(Identified::new(
                        this_chain_id.clone(),
                        Data::TrustedClientState(TrustedClientState {
                            fetched_at: trusted_client_state_fetched_at_height,
                            client_id: trusted_client_state_client_id,
                            trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ChannelEndPath(ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchChannelEnd {
                                at: trusted_client_state_fetched_at_height,
                                port_id: event.port_id.clone(),
                                channel_id: event.channel_id.clone(),
                            },
                        ),
                    ]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ChannelOpenAck(AggregateChannelOpenAck {
                            event_height,
                            event,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::ChannelOpenConfirm(AggregateChannelOpenConfirm {
                event_height,
                event,
            }) => {
                let consensus_state_height = trusted_client_state_fetched_at_height;

                assert_eq!(
                    consensus_state_height.revision_number(),
                    event_height.revision_number(),
                    "{consensus_state_height}, {event_height}",
                );

                assert!(
                    consensus_state_height.revision_height() >= event_height.revision_height(),
                    "{} < {}",
                    consensus_state_height.revision_height(),
                    event_height.revision_height()
                );

                RelayerMsg::Aggregate {
                    data: [AnyLightClientIdentified::from(Identified::new(
                        this_chain_id.clone(),
                        Data::TrustedClientState(TrustedClientState {
                            fetched_at: trusted_client_state_fetched_at_height,
                            client_id: trusted_client_state_client_id,
                            trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchStateProof {
                                at: trusted_client_state_fetched_at_height,
                                path: proof::Path::ChannelEndPath(ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }),
                            },
                        ),
                        fetch::<L>(
                            this_chain_id.clone(),
                            FetchChannelEnd {
                                at: trusted_client_state_fetched_at_height,
                                port_id: event.port_id.clone(),
                                channel_id: event.channel_id.clone(),
                            },
                        ),
                    ]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ChannelOpenConfirm(AggregateChannelOpenConfirm {
                            event_height,
                            event,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::RecvPacket(AggregateRecvPacket {
                event_height,
                event,
            }) => {
                //
                tracing::debug!("building aggregate for RecvPacket");

                RelayerMsg::Aggregate {
                    data: [AnyLightClientIdentified::from(Identified::new(
                        this_chain_id.clone(),
                        Data::TrustedClientState(TrustedClientState {
                            fetched_at: trusted_client_state_fetched_at_height,
                            client_id: trusted_client_state_client_id,
                            trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [fetch::<L>(
                        this_chain_id.clone(),
                        FetchStateProof {
                            at: trusted_client_state_fetched_at_height,
                            path: proof::Path::CommitmentPath(CommitmentPath {
                                port_id: event.packet_src_port.clone(),
                                channel_id: event.packet_src_channel.clone(),
                                sequence: event.packet_sequence,
                            }),
                        },
                    )]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::RecvPacket(AggregateRecvPacket {
                            event_height,
                            event,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::AckPacket(AggregateAckPacket {
                event_height,
                event,
                block_hash,
                counterparty_client_id,
            }) => RelayerMsg::Aggregate {
                data: [AnyLightClientIdentified::from(Identified::new(
                    this_chain_id.clone(),
                    Data::TrustedClientState(TrustedClientState {
                        fetched_at: trusted_client_state_fetched_at_height,
                        client_id: trusted_client_state_client_id,
                        trusted_client_state: trusted_client_state.clone(),
                    }),
                ))]
                .into(),
                queue: [
                    fetch::<L>(
                        this_chain_id.clone(),
                        FetchPacketAcknowledgement {
                            block_hash: block_hash.clone(),
                            destination_port_id: event.packet_dst_port.clone(),
                            destination_channel_id: event.packet_dst_channel.clone(),
                            sequence: event.packet_sequence,
                            __marker: PhantomData,
                        },
                    ),
                    fetch::<L>(
                        this_chain_id.clone(),
                        FetchStateProof {
                            at: trusted_client_state_fetched_at_height,
                            path: proof::Path::AcknowledgementPath(AcknowledgementPath {
                                port_id: event.packet_dst_port.clone(),
                                channel_id: event.packet_dst_channel.clone(),
                                sequence: event.packet_sequence,
                            }),
                        },
                    ),
                ]
                .into(),
                receiver: AnyLightClientIdentified::from(Identified::new(
                    this_chain_id,
                    Aggregate::AckPacket(AggregateAckPacket {
                        event_height,
                        event,
                        block_hash,
                        counterparty_client_id,
                    }),
                )),
            },
        }
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateConnectionOpenTry<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    identified!(ClientStateProof<L>): IsAggregateData,
    identified!(ClientConsensusStateProof<L>): IsAggregateData,
    identified!(ConnectionProof<L>): IsAggregateData,
    identified!(ConnectionEnd<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![
        identified!(TrustedClientState<L>),
        identified!(ClientStateProof<L>),
        identified!(ClientConsensusStateProof<L>),
        identified!(ConnectionProof<L>),
        identified!(ConnectionEnd<L>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateConnectionOpenTry {
                    event_height: trusted_height,
                    event,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: TrustedClientState {
                    fetched_at: _,
                    client_id: _,
                    trusted_client_state
                }
            },
            Identified {
                chain_id: client_state_proof_chain_id,
                data: ClientStateProof {
                    height: client_state_proof_height,
                    proof: client_state_proof,
                }
            },
            Identified {
                chain_id: consensus_state_proof_chain_id,
                data: ClientConsensusStateProof {
                    height: consensus_state_proof_height,
                    proof: consensus_state_proof,
                }
            },
            Identified {
                chain_id: connection_proof_chain_id,
                data: ConnectionProof {
                    height: connection_proof_height,
                    proof: connection_proof,
                }
            },
            Identified {
                chain_id: connection_end_chain_id,
                data: ConnectionEnd(connection_end),
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert!(consensus_state_proof_height.revision_height() >= trusted_height.revision_height());
        assert!(client_state_proof_height.revision_height() >= trusted_height.revision_height());

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);

        assert_eq!(client_state_proof_chain_id, this_chain_id);
        assert_eq!(consensus_state_proof_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);
        assert_eq!(connection_end_chain_id, this_chain_id);

        let consensus_height = trusted_client_state.height();

        msg::<L::Counterparty>(
            counterparty_chain_id,
            MsgConnectionOpenTryData {
                msg: MsgConnectionOpenTry {
                    client_id: event.counterparty_client_id,
                    client_state: trusted_client_state,
                    counterparty: connection::counterparty::Counterparty {
                        client_id: event.client_id,
                        connection_id: event.connection_id,
                        prefix: MerklePrefix {
                            key_prefix: b"ibc".to_vec(),
                        },
                    },
                    delay_period: DELAY_PERIOD,
                    counterparty_versions: connection_end.versions,
                    proof_height: connection_proof_height,
                    proof_init: connection_proof,
                    proof_client: client_state_proof,
                    proof_consensus: consensus_state_proof,
                    consensus_height,
                },
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateConnectionOpenAck<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    identified!(ClientStateProof<L>): IsAggregateData,
    identified!(ClientConsensusStateProof<L>): IsAggregateData,
    identified!(ConnectionProof<L>): IsAggregateData,
    identified!(ConnectionEnd<L>): IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![
        identified!(TrustedClientState<L>),
        identified!(ClientStateProof<L>),
        identified!(ClientConsensusStateProof<L>),
        identified!(ConnectionProof<L>),
        identified!(ConnectionEnd<L>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateConnectionOpenAck {
                    event_height: trusted_height,
                    event,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: TrustedClientState {
                    fetched_at: _,
                    client_id: _,
                    trusted_client_state
                }
            },
            Identified {
                chain_id: client_state_proof_chain_id,
                data: ClientStateProof {
                    height: client_state_proof_height,
                    proof: client_state_proof,
                }
            },
            Identified {
                chain_id: consensus_state_proof_chain_id,
                data: ClientConsensusStateProof {
                    height: consensus_state_proof_height,
                    proof: consensus_state_proof,
                }
            },
            Identified {
                chain_id: connection_proof_chain_id,
                data: ConnectionProof {
                    height: connection_proof_height,
                    proof: connection_proof,
                }
            },
            Identified {
                chain_id: connection_end_chain_id,
                data: ConnectionEnd(connection_end),
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert!(consensus_state_proof_height.revision_height() >= trusted_height.revision_height());
        assert!(client_state_proof_height.revision_height() >= trusted_height.revision_height());

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(client_state_proof_chain_id, this_chain_id);
        assert_eq!(consensus_state_proof_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);
        assert_eq!(connection_end_chain_id, this_chain_id);

        let consensus_height = trusted_client_state.height();

        msg::<L::Counterparty>(
            counterparty_chain_id,
            MsgConnectionOpenAckData {
                msg: MsgConnectionOpenAck {
                    connection_id: event.counterparty_connection_id,
                    counterparty_connection_id: event.connection_id,
                    // TODO: Figure out a way to not panic here, likely by encoding this invariant into the type somehow
                    version: connection_end.versions[0].clone(),
                    client_state: trusted_client_state,
                    proof_height: connection_proof_height.into(),
                    proof_try: connection_proof,
                    proof_client: client_state_proof,
                    proof_consensus: consensus_state_proof,
                    consensus_height: consensus_height.into(),
                },
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateConnectionOpenConfirm<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    identified!(ClientStateProof<L>): IsAggregateData,
    identified!(ClientConsensusStateProof<L>): IsAggregateData,
    identified!(ConnectionProof<L>): IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![
        identified!(TrustedClientState<L>),
        identified!(ConnectionProof<L>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateConnectionOpenConfirm {
                    event_height: _,
                    event,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: TrustedClientState {
                    fetched_at: _,
                    client_id: _,
                    trusted_client_state
                }
            },
            Identified {
                chain_id: connection_proof_chain_id,
                data: ConnectionProof {
                    height: connection_proof_height,
                    proof: connection_proof
                }
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);

        msg::<L::Counterparty>(
            counterparty_chain_id,
            MsgConnectionOpenConfirmData(MsgConnectionOpenConfirm {
                connection_id: event.counterparty_connection_id,
                proof_height: connection_proof_height,
                proof_ack: connection_proof,
            }),
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateChannelOpenTry<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    identified!(ChannelEndProof<L>): IsAggregateData,
    identified!(ConnectionEnd<L>): IsAggregateData,
    identified!(ChannelEnd<L>): IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![
        identified!(TrustedClientState<L>),
        identified!(ChannelEndProof<L>),
        identified!(ConnectionEnd<L>),
        identified!(ChannelEnd<L>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateChannelOpenTry {
                    event_height: _,
                    event,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: TrustedClientState {
                    fetched_at: _,
                    client_id: _,
                    trusted_client_state
                }
            },
            Identified {
                chain_id: channel_proof_chain_id,
                data: ChannelEndProof {
                    proof: channel_proof,
                    height: channel_proof_height
                }
            },
            Identified {
                chain_id: _connection_end_chain_id,
                data: ConnectionEnd(connection)
            },
            Identified {
                chain_id: _channel_end_chain_id,
                data: ChannelEnd {
                    channel,
                    __marker: _
                },
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        assert_eq!(channel_proof_chain_id, this_chain_id);

        msg::<L::Counterparty>(
            counterparty_chain_id,
            MsgChannelOpenTryData {
                msg: MsgChannelOpenTry {
                    port_id: channel.counterparty.port_id.clone(),
                    channel: Channel {
                        state: channel::state::State::Tryopen,
                        ordering: channel.ordering,
                        counterparty: channel::counterparty::Counterparty {
                            port_id: event.port_id.clone(),
                            channel_id: event.channel_id.clone().to_string(),
                        },
                        connection_hops: vec![connection
                            .counterparty
                            .connection_id
                            .parse()
                            .unwrap()],
                        version: event.version.clone(),
                    },
                    // NOTE: Review behaviour here
                    counterparty_version: event.version,
                    proof_init: channel_proof,
                    proof_height: channel_proof_height.into(),
                },
                __marker: PhantomData,
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateChannelOpenAck<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    identified!(ChannelEndProof<L>): IsAggregateData,
    identified!(ChannelEnd<L>): IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![
        identified!(TrustedClientState<L>),
        identified!(ChannelEndProof<L>),
        identified!(ChannelEnd<L>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateChannelOpenAck {
                    event_height: _,
                    event,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: TrustedClientState {
                    fetched_at: _,
                    client_id: _,
                    trusted_client_state
                }
            },
            Identified {
                chain_id: channel_proof_chain_id,
                data: ChannelEndProof {
                    height: channel_proof_height,
                    proof: channel_proof,
                }
            },
            Identified {
                chain_id: channel_end_chain_id,
                data: ChannelEnd {
                    channel,
                    __marker: _
                },
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(channel_proof_chain_id, this_chain_id);
        assert_eq!(channel_end_chain_id, this_chain_id);

        msg::<L::Counterparty>(
            counterparty_chain_id,
            MsgChannelOpenAckData {
                msg: MsgChannelOpenAck {
                    port_id: channel.counterparty.port_id.clone(),
                    channel_id: event.counterparty_channel_id,
                    counterparty_channel_id: event.channel_id,
                    counterparty_version: event.version,
                    proof_try: channel_proof,
                    proof_height: channel_proof_height.into(),
                },
                __marker: PhantomData,
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateChannelOpenConfirm<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    identified!(ChannelEndProof<L>): IsAggregateData,
    identified!(ChannelEnd<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![
        identified!(TrustedClientState<L>),
        identified!(ChannelEndProof<L>),
        identified!(ChannelEnd<L>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateChannelOpenConfirm {
                    event_height: _,
                    event,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: TrustedClientState {
                    fetched_at: _,
                    client_id: _,
                    trusted_client_state
                }
            },
            Identified {
                chain_id: channel_proof_chain_id,
                data: ChannelEndProof {
                    height: channel_proof_height,
                    proof: channel_proof,
                }
            },
            Identified {
                chain_id: channel_end_chain_id,
                data: ChannelEnd {
                    channel,
                    __marker: _
                },
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, channel_proof_chain_id);
        assert_eq!(channel_end_chain_id, this_chain_id);

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        msg::<L::Counterparty>(
            counterparty_chain_id,
            MsgChannelOpenConfirmData {
                msg: MsgChannelOpenConfirm {
                    port_id: channel.counterparty.port_id,
                    channel_id: event.counterparty_channel_id,
                    proof_ack: channel_proof,
                    proof_height: channel_proof_height.into(),
                },
                __marker: PhantomData,
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateRecvPacket<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    identified!(CommitmentProof<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![
        identified!(TrustedClientState<L>),
        identified!(CommitmentProof<L>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateRecvPacket {
                    event_height: _,
                    event,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: TrustedClientState {
                    fetched_at: _,
                    client_id: _,
                    trusted_client_state
                }
            },
            Identified {
                chain_id: commitment_proof_chain_id,
                data: CommitmentProof {
                    height: commitment_proof_height,
                    proof: commitment_proof
                }
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, commitment_proof_chain_id);

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        msg::<L::Counterparty>(
            counterparty_chain_id,
            MsgRecvPacketData {
                msg: MsgRecvPacket {
                    packet: Packet {
                        sequence: event.packet_sequence,
                        source_port: event.packet_src_port,
                        source_channel: event.packet_src_channel,
                        destination_port: event.packet_dst_port,
                        destination_channel: event.packet_dst_channel,
                        data: event.packet_data_hex,
                        timeout_height: event.packet_timeout_height,
                        timeout_timestamp: event.packet_timeout_timestamp,
                    },
                    proof_commitment: commitment_proof,
                    proof_height: commitment_proof_height.into(),
                },
                __marker: PhantomData,
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateAckPacket<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    identified!(PacketAcknowledgement<L>): IsAggregateData,
    identified!(AcknowledgementProof<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L::Counterparty>)>,
{
    type AggregatedData = HList![
        identified!(TrustedClientState<L>),
        identified!(PacketAcknowledgement<L>),
        identified!(AcknowledgementProof<L>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateAckPacket {
                    event_height: _,
                    event,
                    block_hash: _,
                    counterparty_client_id: _,
                },
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: TrustedClientState {
                    fetched_at: _,
                    client_id: _,
                    trusted_client_state
                }
            },
            Identified {
                chain_id: packet_acknowledgement_chain_id,
                data: PacketAcknowledgement { fetched_by: _, ack }
            },
            Identified {
                chain_id: commitment_proof_chain_id,
                data: AcknowledgementProof {
                    proof: acknowledgement_proof,
                    height: acknowledgement_proof_height
                },
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, packet_acknowledgement_chain_id);
        assert_eq!(commitment_proof_chain_id, this_chain_id);

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        msg::<L::Counterparty>(
            counterparty_chain_id,
            MsgAckPacketData {
                msg: MsgAcknowledgement {
                    proof_height: acknowledgement_proof_height.into(),
                    packet: Packet {
                        sequence: event.packet_sequence,
                        source_port: event.packet_src_port,
                        source_channel: event.packet_src_channel,
                        destination_port: event.packet_dst_port,
                        destination_channel: event.packet_dst_channel,
                        data: event.packet_data_hex,
                        timeout_height: event.packet_timeout_height,
                        timeout_timestamp: event.packet_timeout_timestamp,
                    },
                    acknowledgement: ack,
                    proof_acked: acknowledgement_proof,
                },
                __marker: PhantomData,
            },
        )
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateFetchCounterpartyStateProof<L>)
where
    identified!(TrustedClientState<L>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L::Counterparty>)>,
{
    type AggregatedData = HList![identified!(TrustedClientState<L>),];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateFetchCounterpartyStateProof {
                    counterparty_client_id: _,
                    fetch: fetch_,
                },
        }: Self,
        hlist_pat![Identified {
            chain_id: trusted_client_state_chain_id,
            data: TrustedClientState {
                fetched_at: _,
                client_id: _,
                trusted_client_state
            }
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);

        let counterparty_chain_id: ChainIdOf<ChainOf<L::Counterparty>> =
            trusted_client_state.chain_id();

        fetch::<L::Counterparty>(counterparty_chain_id, fetch_)
    }
}

impl<L: LightClient> UseAggregate<L> for identified!(AggregateCreateClient<L>)
where
    identified!(SelfClientState<L::Counterparty>): IsAggregateData,
    identified!(SelfConsensusState<L::Counterparty>): IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<L>)>,
{
    type AggregatedData = HList![
        identified!(SelfClientState<L::Counterparty>),
        identified!(SelfConsensusState<L::Counterparty>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data: this,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: self_client_state_chain_id,
                data: SelfClientState(self_client_state)
            },
            Identified {
                chain_id: self_consensus_state_chain_id,
                data: SelfConsensusState(self_consensus_state)
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(self_client_state_chain_id, self_consensus_state_chain_id);

        // let counterparty_chain_id = self_client_state_chain_id;

        msg::<L>(
            this_chain_id,
            MsgCreateClientData {
                config: this.config,
                msg: MsgCreateClient {
                    client_state: self_client_state,
                    consensus_state: self_consensus_state,
                },
            },
        )
    }
}
