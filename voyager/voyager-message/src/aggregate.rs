use std::{collections::VecDeque, fmt::Display, marker::PhantomData};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use serde::{Deserialize, Serialize};
use unionlabs::{
    events::{
        ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck, ConnectionOpenInit,
        ConnectionOpenTry, RecvPacket, SendPacket,
    },
    hash::H256,
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
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath,
    },
    traits::{ChainIdOf, ClientIdOf, ClientState, HeightOf},
    DELAY_PERIOD,
};

use crate::{
    any_enum,
    data::{
        AnyData, Data, IbcProof, IbcState, PacketAcknowledgement, SelfClientState,
        SelfConsensusState,
    },
    fetch,
    fetch::{
        AnyFetch, Fetch, FetchLatestClientState, FetchPacketAcknowledgement, FetchProof,
        FetchState, FetchUpdateHeaders,
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
    AnyLightClientIdentified, ChainExt, DoAggregate, Identified, RelayerMsg,
};

any_enum! {
    /// Aggregate data, using data from [`AggregateData`]
    #[any = AnyAggregate]
    pub enum Aggregate<Hc: ChainExt, Tr: ChainExt> {
        ConnectionOpenTry(AggregateConnectionOpenTry<Hc, Tr>),
        ConnectionOpenAck(AggregateConnectionOpenAck<Hc, Tr>),
        ConnectionOpenConfirm(AggregateConnectionOpenConfirm<Hc, Tr>),

        ChannelOpenTry(AggregateChannelOpenTry<Hc, Tr>),
        ChannelOpenAck(AggregateChannelOpenAck<Hc, Tr>),
        ChannelOpenConfirm(AggregateChannelOpenConfirm<Hc, Tr>),

        RecvPacket(AggregateRecvPacket<Hc, Tr>),
        AckPacket(AggregateAckPacket<Hc, Tr>),

        ConnectionFetchFromChannelEnd(AggregateConnectionFetchFromChannelEnd<Hc, Tr>),

        // Aggregate that fetches the connection info from the channel
        ChannelHandshakeUpdateClient(AggregateChannelHandshakeUpdateClient<Hc, Tr>),

        PacketUpdateClient(AggregatePacketUpdateClient<Hc, Tr>),

        WaitForTrustedHeight(AggregateWaitForTrustedHeight<Hc, Tr>),

        FetchCounterpartyStateproof(AggregateFetchCounterpartyStateProof<Hc, Tr>),

        UpdateClientFromClientId(AggregateUpdateClientFromClientId<Hc, Tr>),

        UpdateClient(AggregateUpdateClient<Hc, Tr>),
        UpdateClientWithCounterpartyChainIdData(AggregateUpdateClientWithCounterpartyChainId<Hc, Tr>),

        CreateClient(AggregateCreateClient<Hc, Tr>),

        AggregateMsgAfterUpdate(AggregateMsgAfterUpdate<Hc, Tr>),

        LightClientSpecific(LightClientSpecificAggregate<Hc, Tr>),
    }
}

impl<Hc: ChainExt, Tr: ChainExt> identified!(Aggregate<Hc, Tr>) {
    pub fn handle(self, data: VecDeque<AnyLightClientIdentified<AnyData>>) -> Vec<RelayerMsg>
    where
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
        Identified<Tr, Hc, IbcState<Tr, Hc, ClientStatePath<Tr::ClientId>>>: IsAggregateData,

        Identified<Hc, Tr, IbcProof<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,

        Identified<Hc, Tr, IbcProof<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>>:
            IsAggregateData,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ConnectionPath>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ChannelEndPath>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<Hc, Tr, CommitmentPath>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<Hc, Tr, AcknowledgementPath>>: IsAggregateData,

        identified!(SelfClientState<Tr, Hc>): IsAggregateData,
        identified!(SelfConsensusState<Tr, Hc>): IsAggregateData,

        identified!(PacketAcknowledgement<Hc, Tr>): IsAggregateData,

        Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>: IsAggregateData,
        Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>: IsAggregateData,
        Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>: IsAggregateData,

        Identified<Hc, Tr, Hc::Aggregate<Tr>>: DoAggregate,

        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,

        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Hc>)>,

        AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Hc, Tr>)>,
        AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,

        AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
    {
        let chain_id = self.chain_id;

        match self.data {
            Aggregate::ConnectionOpenTry(init) => {
                [do_aggregate(Identified::new(chain_id, init), data)].into()
            }
            Aggregate::ConnectionOpenAck(ack) => {
                [do_aggregate(Identified::new(chain_id, ack), data)].into()
            }
            Aggregate::ConnectionOpenConfirm(confirm) => {
                [do_aggregate(Identified::new(chain_id, confirm), data)].into()
            }
            Aggregate::ChannelOpenTry(try_) => {
                [do_aggregate(Identified::new(chain_id, try_), data)].into()
            }
            Aggregate::ChannelOpenAck(ack) => {
                [do_aggregate(Identified::new(chain_id, ack), data)].into()
            }
            Aggregate::ChannelOpenConfirm(confirm) => {
                [do_aggregate(Identified::new(chain_id, confirm), data)].into()
            }
            Aggregate::UpdateClientFromClientId(update_client) => {
                [do_aggregate(Identified::new(chain_id, update_client), data)].into()
            }
            Aggregate::UpdateClient(update_client) => {
                [do_aggregate(Identified::new(chain_id, update_client), data)].into()
            }
            Aggregate::UpdateClientWithCounterpartyChainIdData(aggregate) => {
                [do_aggregate(Identified::new(chain_id, aggregate), data)].into()
            }
            Aggregate::CreateClient(create_client) => {
                [do_aggregate(Identified::new(chain_id, create_client), data)].into()
            }
            Aggregate::AggregateMsgAfterUpdate(aggregate) => {
                [do_aggregate(Identified::new(chain_id, aggregate), data)].into()
            }
            Aggregate::LightClientSpecific(LightClientSpecificAggregate(aggregate)) => {
                <Identified<_, _, Hc::Aggregate<Tr>> as DoAggregate>::do_aggregate(
                    Identified::new(chain_id, aggregate),
                    data,
                )
            }
            Aggregate::ConnectionFetchFromChannelEnd(aggregate) => {
                [do_aggregate(Identified::new(chain_id, aggregate), data)].into()
            }
            Aggregate::ChannelHandshakeUpdateClient(channel_handshake_update_client) => {
                [do_aggregate(
                    Identified::new(chain_id, channel_handshake_update_client),
                    data,
                )]
                .into()
            }
            Aggregate::PacketUpdateClient(packet_update_client) => [do_aggregate(
                Identified::new(chain_id, packet_update_client),
                data,
            )]
            .into(),
            Aggregate::RecvPacket(recv_packet) => {
                [do_aggregate(Identified::new(chain_id, recv_packet), data)].into()
            }
            Aggregate::AckPacket(ack_packet) => {
                [do_aggregate(Identified::new(chain_id, ack_packet), data)].into()
            }
            Aggregate::WaitForTrustedHeight(agg) => {
                [do_aggregate(Identified::new(chain_id, agg), data)].into()
            }
            Aggregate::FetchCounterpartyStateproof(agg) => {
                [do_aggregate(Identified::new(chain_id, agg), data)].into()
            }
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> Display for Aggregate<Hc, Tr> {
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
pub struct AggregateConnectionOpenTry<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ConnectionOpenInit<ClientIdOf<Hc>, ClientIdOf<Tr>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionOpenAck<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ConnectionOpenTry<ClientIdOf<Hc>, ClientIdOf<Tr>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionOpenConfirm<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ConnectionOpenAck<ClientIdOf<Hc>, ClientIdOf<Tr>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenTry<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ChannelOpenInit,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenAck<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ChannelOpenTry,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenConfirm<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ChannelOpenAck,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateRecvPacket<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: SendPacket,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateAckPacket<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: RecvPacket,
    // HACK: Need to pass the block hash through, figure out a better/cleaner way to do this
    pub block_hash: H256,
    pub counterparty_client_id: ClientIdOf<Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionFetchFromChannelEnd<Hc: ChainExt, Tr: ChainExt> {
    pub at: HeightOf<Hc>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelHandshakeUpdateClient<Hc: ChainExt, Tr: ChainExt> {
    // Will be threaded through to the update msg
    pub update_to: HeightOf<Hc>,
    pub event_height: HeightOf<Hc>,
    pub channel_handshake_event: ChannelHandshakeEvent,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
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
pub struct AggregatePacketUpdateClient<Hc: ChainExt, Tr: ChainExt> {
    // Will be threaded through to the update msg
    pub update_to: HeightOf<Hc>,
    pub event_height: HeightOf<Hc>,
    pub block_hash: H256,
    pub packet_event: PacketEvent,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum PacketEvent {
    Send(SendPacket),
    Recv(RecvPacket),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateFetchCounterpartyStateProof<Hc: ChainExt, Tr: ChainExt> {
    pub counterparty_client_id: ClientIdOf<Tr>,
    pub fetch: FetchProof<Tr, Hc>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClientFromClientId<Hc: ChainExt, Tr: ChainExt> {
    pub client_id: ClientIdOf<Hc>,
    pub counterparty_client_id: ClientIdOf<Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClient<Hc: ChainExt, Tr: ChainExt> {
    pub update_to: HeightOf<Hc>,
    pub client_id: ClientIdOf<Hc>,
    pub counterparty_client_id: ClientIdOf<Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateWaitForTrustedHeight<Hc: ChainExt, Tr: ChainExt> {
    pub wait_for: HeightOf<Hc>,
    pub client_id: ClientIdOf<Hc>,
    pub counterparty_client_id: ClientIdOf<Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClientWithCounterpartyChainId<Hc: ChainExt, Tr: ChainExt> {
    pub update_to: HeightOf<Hc>,
    pub client_id: ClientIdOf<Hc>,
    pub counterparty_client_id: ClientIdOf<Tr>,
    pub counterparty_chain_id: ChainIdOf<Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateMsgUpdateClient<Hc: ChainExt, Tr: ChainExt> {
    pub update_to: HeightOf<Hc>,
    pub client_id: ClientIdOf<Hc>,
    pub counterparty_client_id: ClientIdOf<Tr>,
    pub counterparty_chain_id: ChainIdOf<Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateCreateClient<Hc: ChainExt, Tr: ChainExt> {
    pub config: <Hc as ChainExt>::Config,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientSpecificAggregate<Hc: ChainExt, Tr: ChainExt>(pub Hc::Aggregate<Tr>);

/// Messages that will be re-queued after an update.
#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum AggregateMsgAfterUpdate<Hc: ChainExt, Tr: ChainExt> {
    ConnectionOpenTry(AggregateConnectionOpenTry<Hc, Tr>),
    ConnectionOpenAck(AggregateConnectionOpenAck<Hc, Tr>),
    ConnectionOpenConfirm(AggregateConnectionOpenConfirm<Hc, Tr>),

    ChannelOpenTry(AggregateChannelOpenTry<Hc, Tr>),
    ChannelOpenAck(AggregateChannelOpenAck<Hc, Tr>),
    ChannelOpenConfirm(AggregateChannelOpenConfirm<Hc, Tr>),

    RecvPacket(AggregateRecvPacket<Hc, Tr>),
    AckPacket(AggregateAckPacket<Hc, Tr>),
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateChannelHandshakeUpdateClient<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData = HList![Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateChannelHandshakeUpdateClient {
                    update_to,
                    channel_handshake_event,
                    event_height,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: IbcState {
                path: _,
                height: _,
                state: connection,
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);

        let event_msg = match channel_handshake_event {
            ChannelHandshakeEvent::Init(init) => {
                AggregateMsgAfterUpdate::ChannelOpenTry(AggregateChannelOpenTry {
                    event_height,
                    event: init,
                    __marker: PhantomData,
                })
            }
            ChannelHandshakeEvent::Try(try_) => {
                AggregateMsgAfterUpdate::ChannelOpenAck(AggregateChannelOpenAck {
                    event_height,
                    event: try_,
                    __marker: PhantomData,
                })
            }
            ChannelHandshakeEvent::Ack(ack) => {
                AggregateMsgAfterUpdate::ChannelOpenConfirm(AggregateChannelOpenConfirm {
                    event_height,
                    event: ack,
                    __marker: PhantomData,
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

pub fn mk_aggregate_wait_for_update<Hc: ChainExt, Tr: ChainExt>(
    chain_id: ChainIdOf<Hc>,
    client_id: ClientIdOf<Hc>,
    counterparty_client_id: ClientIdOf<Tr>,
    wait_for: HeightOf<Hc>,
) -> RelayerMsg
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    RelayerMsg::Aggregate {
        queue: [fetch::<Hc, Tr>(
            chain_id.clone(),
            FetchLatestClientState {
                path: ClientStatePath {
                    client_id: client_id.clone(),
                },
                __marker: PhantomData,
            },
        )]
        .into(),
        data: [].into(),
        receiver: AnyLightClientIdentified::from(Identified::new(
            chain_id,
            Aggregate::<Hc, Tr>::WaitForTrustedHeight(AggregateWaitForTrustedHeight {
                wait_for,
                client_id,
                counterparty_client_id,
            }),
        )),
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregatePacketUpdateClient<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData = HList![Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregatePacketUpdateClient {
                    update_to,
                    event_height,
                    block_hash,
                    packet_event,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: IbcState {
                path: _,
                height: _,
                state: connection,
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);

        let event = match packet_event {
            PacketEvent::Send(send) => Aggregate::AggregateMsgAfterUpdate(
                AggregateMsgAfterUpdate::RecvPacket(AggregateRecvPacket {
                    event_height,
                    event: send,
                    __marker: PhantomData,
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
            queue: [fetch::<Hc, Tr>(
                this_chain_id.clone().clone(),
                FetchLatestClientState {
                    path: ClientStatePath {
                        client_id: connection.client_id.clone(),
                    },
                    __marker: PhantomData,
                },
            )]
            .into(),
            data: [].into(),
            receiver: AnyLightClientIdentified::from(Identified::new(
                this_chain_id.clone(),
                Aggregate::<Hc, Tr>::WaitForTrustedHeight(AggregateWaitForTrustedHeight {
                    wait_for: update_to,
                    client_id: connection.client_id.clone(),
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateConnectionFetchFromChannelEnd<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
{
    type AggregatedData = HList![Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data: AggregateConnectionFetchFromChannelEnd { at, __marker: _ },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: IbcState {
                path: _,
                height: _,
                state: channel,
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);

        fetch(
            this_chain_id,
            FetchState {
                at,
                path: ConnectionPath {
                    connection_id: channel.connection_hops[0].clone(),
                }
                .into(),
            },
        )
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateUpdateClientFromClientId<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateUpdateClientFromClientId {
                    client_id,
                    counterparty_client_id,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: IbcState {
                path: ClientStatePath {
                    client_id: trusted_client_state_client_id
                },
                height: trusted_client_state_fetched_at_height,
                state: trusted_client_state
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);
        assert_eq!(trusted_client_state_client_id, client_id);

        let counterparty_chain_id = trusted_client_state.chain_id();

        RelayerMsg::Aggregate {
            queue: [fetch::<Tr, Hc>(
                counterparty_chain_id.clone(),
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
                FetchLatestClientState {
                    path: ClientStatePath {
                        client_id: counterparty_client_id.clone(),
                    },
                    __marker: PhantomData,
                },
            )]
            .into(),
            data: [].into(),
            receiver: AnyLightClientIdentified::from(Identified::new(
                this_chain_id,
                Aggregate::UpdateClientWithCounterpartyChainIdData(
                    AggregateUpdateClientWithCounterpartyChainId {
                        update_to: trusted_client_state_fetched_at_height,
                        client_id,
                        counterparty_client_id,
                        counterparty_chain_id,
                    },
                ),
            )),
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateUpdateClient<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateUpdateClient {
                    update_to,
                    client_id: update_client_id,
                    counterparty_client_id: update_counterparty_client_id,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: IbcState {
                path: ClientStatePath {
                    client_id: trusted_client_state_client_id
                },
                height: _trusted_client_state_fetched_at_height,
                state: trusted_client_state
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, self_chain_id);
        assert_eq!(update_client_id, trusted_client_state_client_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        RelayerMsg::Aggregate {
            queue: [fetch::<Tr, Hc>(
                counterparty_chain_id.clone(),
                FetchLatestClientState {
                    path: ClientStatePath {
                        client_id: update_counterparty_client_id.clone(),
                    },
                    __marker: PhantomData,
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateUpdateClientWithCounterpartyChainId<Hc, Tr>)
where
    Identified<Tr, Hc, IbcState<Tr, Hc, ClientStatePath<Tr::ClientId>>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData =
        HList![Identified<Tr, Hc, IbcState<Tr, Hc, ClientStatePath<Tr::ClientId>>>];

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
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: counterparty_chain_id,
            data: IbcState {
                path: ClientStatePath {
                    client_id: trusted_client_state_client_id
                },
                height: _trusted_client_state_fetched_at_height,
                state: trusted_client_state
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        let self_chain_id: ChainIdOf<Hc> = trusted_client_state.chain_id();

        assert_eq!(this_chain_id, self_chain_id);
        assert_eq!(
            trusted_client_state_client_id,
            update_counterparty_client_id
        );
        assert_eq!(counterparty_chain_id, update_counterparty_chain_id);

        fetch::<Hc, Tr>(
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateWaitForTrustedHeight<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Hc>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateWaitForTrustedHeight {
                    wait_for,
                    client_id,
                    counterparty_client_id,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: trusted_client_state_chain_id,
            data: IbcState {
                path: ClientStatePath {
                    client_id: trusted_client_state_client_id
                },
                height: _trusted_client_state_fetched_at_height,
                state: trusted_client_state
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(trusted_client_state_client_id, client_id);
        assert_eq!(trusted_client_state_chain_id, this_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        tracing::debug!("building WaitForTrustedHeight");

        wait::<Tr, Hc>(
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateMsgAfterUpdate<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data: msg_to_aggregate,
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            data: IbcState {
                path: ClientStatePath {
                    client_id: trusted_client_state_client_id
                },
                height: trusted_client_state_fetched_at_height,
                state: trusted_client_state
            },
            __marker: _,
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
                        Data::ClientState(IbcState::<Hc, Tr, ClientStatePath<Hc::ClientId>> {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ClientStatePath {
                                    client_id: event.client_id.clone(),
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ClientConsensusStatePath {
                                    client_id: event.client_id.clone(),
                                    height: trusted_client_state_height,
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }
                                .into(),
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
                        Data::ClientState(IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ClientStatePath {
                                    client_id: event.client_id.clone(),
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ClientConsensusStatePath {
                                    client_id: event.client_id.clone(),
                                    height: trusted_client_state_height,
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }
                                .into(),
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
                        Data::ClientState(IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [fetch::<Hc, Tr>(
                        this_chain_id.clone(),
                        FetchProof {
                            at: trusted_client_state_fetched_at_height,
                            path: ConnectionPath {
                                connection_id: event.connection_id.clone(),
                            }
                            .into(),
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
                __marker: _,
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
                        Data::ClientState(IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        RelayerMsg::Aggregate {
                            data: [].into(),
                            queue: [fetch::<Hc, Tr>(
                                this_chain_id.clone(),
                                FetchState {
                                    at: trusted_client_state_fetched_at_height,
                                    path: ChannelEndPath {
                                        port_id: event.port_id.clone(),
                                        channel_id: event.channel_id.clone(),
                                    }
                                    .into(),
                                },
                            )]
                            .into(),
                            receiver: AnyLightClientIdentified::from(Identified::new(
                                this_chain_id.clone(),
                                Aggregate::ConnectionFetchFromChannelEnd(
                                    AggregateConnectionFetchFromChannelEnd {
                                        at: trusted_client_state_fetched_at_height,
                                        __marker: PhantomData,
                                    },
                                ),
                            )),
                        },
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        ),
                    ]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ChannelOpenTry(AggregateChannelOpenTry {
                            event_height,
                            event,
                            __marker: PhantomData,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::ChannelOpenAck(AggregateChannelOpenAck {
                event_height,
                event,
                __marker: _,
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
                        Data::ClientState(IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        ),
                    ]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ChannelOpenAck(AggregateChannelOpenAck {
                            event_height,
                            event,
                            __marker: PhantomData,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::ChannelOpenConfirm(AggregateChannelOpenConfirm {
                event_height,
                event,
                __marker: _,
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
                        Data::ClientState(IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        ),
                        fetch::<Hc, Tr>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        ),
                    ]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::ChannelOpenConfirm(AggregateChannelOpenConfirm {
                            event_height,
                            event,
                            __marker: PhantomData,
                        }),
                    )),
                }
            }
            AggregateMsgAfterUpdate::RecvPacket(AggregateRecvPacket {
                event_height,
                event,
                __marker: _,
            }) => {
                //
                tracing::debug!("building aggregate for RecvPacket");

                RelayerMsg::Aggregate {
                    data: [AnyLightClientIdentified::from(Identified::new(
                        this_chain_id.clone(),
                        Data::ClientState(IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        }),
                    ))]
                    .into(),
                    queue: [fetch::<Hc, Tr>(
                        this_chain_id.clone(),
                        FetchProof {
                            at: trusted_client_state_fetched_at_height,
                            path: CommitmentPath {
                                port_id: event.packet_src_port.clone(),
                                channel_id: event.packet_src_channel.clone(),
                                sequence: event.packet_sequence,
                            }
                            .into(),
                        },
                    )]
                    .into(),
                    receiver: AnyLightClientIdentified::from(Identified::new(
                        this_chain_id,
                        Aggregate::RecvPacket(AggregateRecvPacket {
                            event_height,
                            event,
                            __marker: PhantomData,
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
                    Data::ClientState(IbcState {
                        path: ClientStatePath {
                            client_id: trusted_client_state_client_id,
                        },
                        height: trusted_client_state_fetched_at_height,
                        state: trusted_client_state,
                    }),
                ))]
                .into(),
                queue: [
                    fetch::<Hc, Tr>(
                        this_chain_id.clone(),
                        FetchPacketAcknowledgement {
                            block_hash: block_hash.clone(),
                            destination_port_id: event.packet_dst_port.clone(),
                            destination_channel_id: event.packet_dst_channel.clone(),
                            sequence: event.packet_sequence,
                            __marker: PhantomData,
                        },
                    ),
                    fetch::<Hc, Tr>(
                        this_chain_id.clone(),
                        FetchProof {
                            at: trusted_client_state_fetched_at_height,
                            path: AcknowledgementPath {
                                port_id: event.packet_dst_port.clone(),
                                channel_id: event.packet_dst_channel.clone(),
                                sequence: event.packet_sequence,
                            }
                            .into(),
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateConnectionOpenTry<Hc, Tr>)
where
    // state
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>: IsAggregateData,

    // proof
    Identified<Hc, Tr, IbcProof<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ConnectionPath>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ConnectionPath>>,
        Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateConnectionOpenTry {
                    event_height: trusted_height,
                    event,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: IbcState {
                    path: ClientStatePath {
                        client_id: _trusted_client_state_client_id
                    },
                    height: _trusted_client_state_fetched_at_height,
                    state: trusted_client_state
                },
                __marker: _,
            },
            Identified {
                chain_id: client_state_proof_chain_id,
                data: IbcProof {
                    height: client_state_proof_height,
                    proof: client_state_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: consensus_state_proof_chain_id,
                data: IbcProof {
                    height: consensus_state_proof_height,
                    proof: consensus_state_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: connection_proof_chain_id,
                data: IbcProof {
                    height: connection_proof_height,
                    proof: connection_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: connection_end_chain_id,
                data: IbcState {
                    path: _,
                    height: _,
                    state: connection_end
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert!(consensus_state_proof_height.revision_height() >= trusted_height.revision_height());
        assert!(client_state_proof_height.revision_height() >= trusted_height.revision_height());

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);

        assert_eq!(client_state_proof_chain_id, this_chain_id);
        assert_eq!(consensus_state_proof_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);
        assert_eq!(connection_end_chain_id, this_chain_id);

        let consensus_height = trusted_client_state.height();

        msg::<Tr, Hc>(
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateConnectionOpenAck<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ConnectionPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ConnectionPath>>,
        Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateConnectionOpenAck {
                    event_height: trusted_height,
                    event,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: IbcState {
                    path: ClientStatePath {
                        client_id: _trusted_client_state_client_id
                    },
                    height: _trusted_client_state_fetched_at_height,
                    state: trusted_client_state
                },
                __marker: _
            },
            Identified {
                chain_id: client_state_proof_chain_id,
                data: IbcProof {
                    height: client_state_proof_height,
                    proof: client_state_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: consensus_state_proof_chain_id,
                data: IbcProof {
                    height: consensus_state_proof_height,
                    proof: consensus_state_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: connection_proof_chain_id,
                data: IbcProof {
                    height: connection_proof_height,
                    proof: connection_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: connection_end_chain_id,
                data: IbcState {
                    path: _,
                    height: _,
                    state: connection_end
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert!(consensus_state_proof_height.revision_height() >= trusted_height.revision_height());
        assert!(client_state_proof_height.revision_height() >= trusted_height.revision_height());

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(client_state_proof_chain_id, this_chain_id);
        assert_eq!(consensus_state_proof_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);
        assert_eq!(connection_end_chain_id, this_chain_id);

        let consensus_height = trusted_client_state.height();

        msg::<Tr, Hc>(
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
                __marker: PhantomData,
            },
        )
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateConnectionOpenConfirm<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ConnectionPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ConnectionPath>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateConnectionOpenConfirm {
                    event_height: _,
                    event,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: IbcState {
                    path: ClientStatePath {
                        client_id: _trusted_client_state_client_id
                    },
                    height: _trusted_client_state_fetched_at_height,
                    state: trusted_client_state
                },
                __marker: _
            },
            Identified {
                chain_id: connection_proof_chain_id,
                data: IbcProof {
                    height: connection_proof_height,
                    proof: connection_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);

        msg::<Tr, Hc>(
            counterparty_chain_id,
            MsgConnectionOpenConfirmData {
                msg: MsgConnectionOpenConfirm {
                    connection_id: event.counterparty_connection_id,
                    proof_height: connection_proof_height,
                    proof_ack: connection_proof,
                },
                __marker: PhantomData,
            },
        )
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateChannelOpenTry<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ChannelEndPath>>,
        Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>,
        Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateChannelOpenTry {
                    event_height: _,
                    event,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: IbcState {
                    path: ClientStatePath {
                        client_id: _trusted_client_state_client_id
                    },
                    height: _trusted_client_state_fetched_at_height,
                    state: trusted_client_state
                },
                __marker: _
            },
            Identified {
                chain_id: channel_proof_chain_id,
                data: IbcProof {
                    proof: channel_proof,
                    height: channel_proof_height,
                    path: _,
                    __marker: _,
                },
                __marker: _
            },
            Identified {
                chain_id: _connection_end_chain_id,
                data: IbcState {
                    path: _,
                    height: _,
                    state: connection,
                },

                __marker: _
            },
            Identified {
                chain_id: _channel_end_chain_id,
                data: IbcState {
                    path: _,
                    height: _,
                    state: channel,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(channel_proof_chain_id, this_chain_id);

        msg::<Tr, Hc>(
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateChannelOpenAck<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ChannelEndPath>>,
        Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateChannelOpenAck {
                    event_height: _,
                    event,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: IbcState {
                    path: ClientStatePath {
                        client_id: _trusted_client_state_client_id
                    },
                    height: _trusted_client_state_fetched_at_height,
                    state: trusted_client_state
                },
                __marker: _
            },
            Identified {
                chain_id: channel_proof_chain_id,
                data: IbcProof {
                    height: channel_proof_height,
                    proof: channel_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _
            },
            Identified {
                chain_id: channel_end_chain_id,
                data: IbcState {
                    path: _,
                    height: _,
                    state: channel,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(channel_proof_chain_id, this_chain_id);
        assert_eq!(channel_end_chain_id, this_chain_id);

        msg::<Tr, Hc>(
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateChannelOpenConfirm<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, ChannelEndPath>>,
        Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateChannelOpenConfirm {
                    event_height: _,
                    event,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: IbcState {
                    path: ClientStatePath {
                        client_id: _trusted_client_state_client_id
                    },
                    height: _trusted_client_state_fetched_at_height,
                    state: trusted_client_state
                },
                __marker: _
            },
            Identified {
                chain_id: channel_proof_chain_id,
                data: IbcProof {
                    height: channel_proof_height,
                    proof: channel_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _
            },
            Identified {
                chain_id: channel_end_chain_id,
                data: IbcState {
                    path: _,
                    height: _,
                    state: channel,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, channel_proof_chain_id);
        assert_eq!(channel_end_chain_id, this_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        msg::<Tr, Hc>(
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateRecvPacket<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, CommitmentPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        Identified<Hc, Tr, IbcProof<Hc, Tr, CommitmentPath>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateRecvPacket {
                    event_height: _,
                    event,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: IbcState {
                    path: ClientStatePath {
                        client_id: _trusted_client_state_client_id
                    },
                    height: _trusted_client_state_fetched_at_height,
                    state: trusted_client_state
                },
                __marker: _
            },
            Identified {
                chain_id: commitment_proof_chain_id,
                data: IbcProof {
                    height: commitment_proof_height,
                    proof: commitment_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, commitment_proof_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        msg::<Tr, Hc>(
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateAckPacket<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, AcknowledgementPath>>: IsAggregateData,
    identified!(PacketAcknowledgement<Hc, Tr>): IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, AcknowledgementPath>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,
        identified!(PacketAcknowledgement<Hc, Tr>),
        Identified<Hc, Tr, IbcProof<Hc, Tr, AcknowledgementPath>>,
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
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                data: IbcState {
                    path: ClientStatePath {
                        client_id: _trusted_client_state_client_id
                    },
                    height: _trusted_client_state_fetched_at_height,
                    state: trusted_client_state
                },
                __marker: _
            },
            Identified {
                chain_id: packet_acknowledgement_chain_id,
                data: PacketAcknowledgement { fetched_by: _, ack },
                __marker: _,
            },
            Identified {
                chain_id: commitment_proof_chain_id,
                data: IbcProof {
                    proof: acknowledgement_proof,
                    height: acknowledgement_proof_height,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, packet_acknowledgement_chain_id);
        assert_eq!(commitment_proof_chain_id, this_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        msg::<Tr, Hc>(
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateFetchCounterpartyStateProof<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>,];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data:
                AggregateFetchCounterpartyStateProof {
                    counterparty_client_id: _,
                    fetch: fetch_,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: trusted_client_state_chain_id,
            data: IbcState {
                height: _,
                path: _,
                state: trusted_client_state,
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        fetch::<Tr, Hc>(counterparty_chain_id, fetch_)
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate for identified!(AggregateCreateClient<Hc, Tr>)
where
    identified!(SelfClientState<Tr, Hc>): IsAggregateData,
    identified!(SelfConsensusState<Tr, Hc>): IsAggregateData,
    AnyLightClientIdentified<AnyMsg>: From<identified!(Msg<Hc, Tr>)>,
{
    type AggregatedData = HList![
        identified!(SelfClientState<Tr, Hc>),
        identified!(SelfConsensusState<Tr, Hc>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            data: this,
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: self_client_state_chain_id,
                data: SelfClientState {
                    self_client_state,
                    __marker: _,
                },
                __marker: _
            },
            Identified {
                chain_id: self_consensus_state_chain_id,
                data: SelfConsensusState {
                    self_consensus_state,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> RelayerMsg {
        assert_eq!(self_client_state_chain_id, self_consensus_state_chain_id);

        // let counterparty_chain_id = self_client_state_chain_id;

        msg::<Hc, Tr>(
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
