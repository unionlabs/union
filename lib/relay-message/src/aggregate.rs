use std::{collections::VecDeque, marker::PhantomData};

use frunk::{hlist_pat, HList};
use macros::apply;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    effect, fetch, queue_msg, wait, HandleAggregate, QueueError, QueueMsg, QueueMsgTypes,
};
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
    any_enum, any_lc,
    data::{
        AnyData, Data, IbcProof, IbcState, LatestHeight, PacketAcknowledgement, SelfClientState,
        SelfConsensusState,
    },
    effect::{
        AnyEffect, Effect, MsgAckPacketData, MsgChannelOpenAckData, MsgChannelOpenConfirmData,
        MsgChannelOpenTryData, MsgConnectionOpenAckData, MsgConnectionOpenConfirmData,
        MsgConnectionOpenTryData, MsgCreateClientData, MsgRecvPacketData,
    },
    fetch::{
        AnyFetch, Fetch, FetchLatestClientState, FetchLatestHeight, FetchPacketAcknowledgement,
        FetchProof, FetchState, FetchUpdateHeaders,
    },
    id, identified,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForTrustedHeight},
    AnyLightClientIdentified, ChainExt, DoAggregate, Identified, RelayMessageTypes,
};

#[apply(any_enum)]
/// Aggregate data, using data from [`AggregateData`]
#[any = AnyAggregate]
#[specific = LightClientSpecificAggregate]
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

    /// Aggregate that fetches the connection info from the channel, requeueing [`Self::AggregateMsgAfterUpdate`]
    ChannelHandshakeMsgAfterUpdate(AggregateChannelHandshakeMsgAfterUpdate<Hc, Tr>),

    PacketUpdateClient(AggregatePacketMsgAfterUpdate<Hc, Tr>),

    WaitForTrustedHeight(AggregateWaitForTrustedHeight<Hc, Tr>),

    FetchCounterpartyStateproof(AggregateFetchCounterpartyStateProof<Hc, Tr>),

    UpdateClient(AggregateUpdateClient<Hc, Tr>),
    UpdateClientFromHeight(AggregateUpdateClientFromHeight<Hc, Tr>),

    CreateClient(AggregateCreateClient<Hc, Tr>),

    AggregateMsgAfterUpdate(AggregateMsgAfterUpdate<Hc, Tr>),

    #[serde(untagged)]
    LightClientSpecific(LightClientSpecificAggregate<Hc, Tr>),
}

impl HandleAggregate<RelayMessageTypes> for AnyLightClientIdentified<AnyAggregate> {
    fn handle(
        self,
        data: VecDeque<<RelayMessageTypes as QueueMsgTypes>::Data>,
    ) -> Result<QueueMsg<RelayMessageTypes>, QueueError> {
        let aggregate = self;

        any_lc! {
            |aggregate| Ok(aggregate.handle(data))
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> identified!(Aggregate<Hc, Tr>) {
    pub fn handle(
        self,
        data: VecDeque<AnyLightClientIdentified<AnyData>>,
    ) -> QueueMsg<RelayMessageTypes>
    where
        identified!(SelfClientState<Tr, Hc>): IsAggregateData,
        identified!(SelfConsensusState<Tr, Hc>): IsAggregateData,

        identified!(LatestHeight<Tr, Hc>): IsAggregateData,

        identified!(PacketAcknowledgement<Hc, Tr>): IsAggregateData,

        // state
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
        Identified<Tr, Hc, IbcState<ClientStatePath<Tr::ClientId>, Tr, Hc>>: IsAggregateData,
        Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,

        // proof
        Identified<Hc, Tr, IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>:
            IsAggregateData,
        Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<CommitmentPath, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<AcknowledgementPath, Hc, Tr>>: IsAggregateData,

        Identified<Hc, Tr, Hc::Aggregate<Tr>>: DoAggregate,

        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,

        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Hc>)>,

        AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Hc, Tr>)>,
        AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,

        AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,

        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
    {
        let chain_id = self.chain_id;

        match self.t {
            Aggregate::ConnectionOpenTry(init) => do_aggregate(id(chain_id, init), data),
            Aggregate::ConnectionOpenAck(ack) => do_aggregate(id(chain_id, ack), data),
            Aggregate::ConnectionOpenConfirm(confirm) => do_aggregate(id(chain_id, confirm), data),
            Aggregate::ChannelOpenTry(try_) => do_aggregate(id(chain_id, try_), data),
            Aggregate::ChannelOpenAck(ack) => do_aggregate(id(chain_id, ack), data),
            Aggregate::ChannelOpenConfirm(confirm) => do_aggregate(id(chain_id, confirm), data),
            Aggregate::UpdateClient(update_client) => {
                do_aggregate(id(chain_id, update_client), data)
            }
            Aggregate::UpdateClientFromHeight(update_client) => {
                do_aggregate(id(chain_id, update_client), data)
            }
            Aggregate::CreateClient(create_client) => {
                do_aggregate(id(chain_id, create_client), data)
            }
            Aggregate::AggregateMsgAfterUpdate(aggregate) => {
                do_aggregate(id(chain_id, aggregate), data)
            }
            Aggregate::LightClientSpecific(LightClientSpecificAggregate(aggregate)) => {
                <Identified<_, _, Hc::Aggregate<Tr>> as DoAggregate>::do_aggregate(
                    id(chain_id, aggregate),
                    data,
                )
            }
            Aggregate::ConnectionFetchFromChannelEnd(aggregate) => {
                do_aggregate(id(chain_id, aggregate), data)
            }
            Aggregate::ChannelHandshakeMsgAfterUpdate(channel_handshake_update_client) => {
                do_aggregate(id(chain_id, channel_handshake_update_client), data)
            }
            Aggregate::PacketUpdateClient(packet_update_client) => {
                do_aggregate(id(chain_id, packet_update_client), data)
            }
            Aggregate::RecvPacket(recv_packet) => do_aggregate(id(chain_id, recv_packet), data),
            Aggregate::AckPacket(ack_packet) => do_aggregate(id(chain_id, ack_packet), data),
            Aggregate::WaitForTrustedHeight(agg) => do_aggregate(id(chain_id, agg), data),
            Aggregate::FetchCounterpartyStateproof(agg) => do_aggregate(id(chain_id, agg), data),
        }
    }
}

#[queue_msg]
pub struct AggregateConnectionOpenTry<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ConnectionOpenInit<ClientIdOf<Hc>, ClientIdOf<Tr>>,
}

#[queue_msg]
pub struct AggregateConnectionOpenAck<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ConnectionOpenTry<ClientIdOf<Hc>, ClientIdOf<Tr>>,
}

#[queue_msg]
pub struct AggregateConnectionOpenConfirm<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ConnectionOpenAck<ClientIdOf<Hc>, ClientIdOf<Tr>>,
}

#[queue_msg]
pub struct AggregateChannelOpenTry<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ChannelOpenInit,
}

#[queue_msg]
pub struct AggregateChannelOpenAck<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ChannelOpenTry,
}

#[queue_msg]
pub struct AggregateChannelOpenConfirm<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: ChannelOpenAck,
}

#[queue_msg]
pub struct AggregateRecvPacket<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: SendPacket,
}

#[queue_msg]
pub struct AggregateAckPacket<Hc: ChainExt, Tr: ChainExt> {
    pub event_height: HeightOf<Hc>,
    pub event: RecvPacket,
    // HACK: Need to pass the block hash through, figure out a better/cleaner way to do this
    // TODO: Replace with the ack directly?
    pub tx_hash: H256,
    pub counterparty_client_id: ClientIdOf<Tr>,
}

#[queue_msg]
pub struct AggregateConnectionFetchFromChannelEnd<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub at: HeightOf<Hc>,
}

#[queue_msg]
pub struct AggregateChannelHandshakeMsgAfterUpdate<Hc: ChainExt, #[cover] Tr: ChainExt> {
    // Will be threaded through to the update msg
    pub event_height: HeightOf<Hc>,
    pub channel_handshake_event: ChannelHandshakeEvent,
}

#[queue_msg]
pub enum ChannelHandshakeEvent {
    Init(ChannelOpenInit),
    Try(ChannelOpenTry),
    Ack(ChannelOpenAck),
}

#[queue_msg]
pub struct AggregatePacketMsgAfterUpdate<Hc: ChainExt, #[cover] Tr: ChainExt> {
    // Will be threaded through to the update msg
    pub update_to: HeightOf<Hc>,
    pub event_height: HeightOf<Hc>,
    pub tx_hash: H256,
    pub packet_event: PacketEvent,
}

#[queue_msg]
pub enum PacketEvent {
    Send(SendPacket),
    Recv(RecvPacket),
}

#[queue_msg]
pub struct AggregateFetchCounterpartyStateProof<Hc: ChainExt, Tr: ChainExt> {
    pub counterparty_client_id: ClientIdOf<Tr>,
    pub fetch: FetchProof<Tr, Hc>,
}

#[queue_msg]
pub struct AggregateUpdateClient<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub client_id: ClientIdOf<Hc>,
}

#[queue_msg]
pub struct AggregateUpdateClientFromHeight<Hc: ChainExt, Tr: ChainExt> {
    pub from_height: HeightOf<Tr>,
    pub client_id: ClientIdOf<Hc>,
}

#[queue_msg]
pub struct AggregateWaitForTrustedHeight<Hc: ChainExt, Tr: ChainExt> {
    pub wait_for: HeightOf<Hc>,
    pub client_id: ClientIdOf<Hc>,
    pub counterparty_client_id: ClientIdOf<Tr>,
}

#[queue_msg]
pub struct AggregateCreateClient<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub config: <Hc as ChainExt>::Config,
}

#[queue_msg]
pub struct LightClientSpecificAggregate<Hc: ChainExt, Tr: ChainExt>(pub Hc::Aggregate<Tr>);

/// Messages that will be re-queued after an update.
#[queue_msg]
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

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateChannelHandshakeMsgAfterUpdate<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData = HList![Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateChannelHandshakeMsgAfterUpdate {
                    channel_handshake_event,
                    event_height,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            t: IbcState {
                path: _,
                height: _,
                state: connection,
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
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

        aggregate(
            [mk_aggregate_wait_for_update(
                this_chain_id.clone(),
                connection.client_id,
                connection.counterparty.client_id,
                event_height,
            )],
            [],
            id(this_chain_id, event_msg),
        )
    }
}

pub fn mk_aggregate_wait_for_update<Hc: ChainExt, Tr: ChainExt>(
    chain_id: ChainIdOf<Hc>,
    client_id: ClientIdOf<Hc>,
    counterparty_client_id: ClientIdOf<Tr>,
    wait_for: HeightOf<Hc>,
) -> QueueMsg<RelayMessageTypes>
where
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    aggregate(
        [fetch(id::<Hc, Tr, _>(
            chain_id.clone(),
            FetchLatestClientState {
                path: ClientStatePath {
                    client_id: client_id.clone(),
                },
                __marker: PhantomData,
            },
        ))],
        [],
        id(
            chain_id,
            AggregateWaitForTrustedHeight::<Hc, Tr> {
                wait_for,
                client_id,
                counterparty_client_id,
            },
        ),
    )
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregatePacketMsgAfterUpdate<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData = HList![Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregatePacketMsgAfterUpdate {
                    update_to,
                    event_height,
                    tx_hash,
                    packet_event,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            t: IbcState {
                path: _,
                height: _,
                state: connection,
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
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
                    tx_hash,
                    counterparty_client_id: connection.counterparty.client_id.clone(),
                }),
            ),
        };

        aggregate(
            [aggregate(
                [fetch(id::<Hc, Tr, _>(
                    this_chain_id.clone().clone(),
                    FetchLatestClientState {
                        path: ClientStatePath {
                            client_id: connection.client_id.clone(),
                        },
                        __marker: PhantomData,
                    },
                ))],
                [],
                id(
                    this_chain_id.clone(),
                    AggregateWaitForTrustedHeight::<Hc, Tr> {
                        wait_for: update_to,
                        client_id: connection.client_id.clone(),
                        counterparty_client_id: connection.counterparty.client_id.clone(),
                    },
                ),
            )],
            [],
            id(this_chain_id, event),
        )
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateConnectionFetchFromChannelEnd<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
{
    type AggregatedData = HList![Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t: AggregateConnectionFetchFromChannelEnd { at, __marker: _ },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            t: IbcState {
                path: _,
                height: _,
                state: channel,
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(this_chain_id, self_chain_id);

        fetch(id::<Hc, Tr, _>(
            this_chain_id,
            FetchState {
                at,
                path: ConnectionPath {
                    connection_id: channel.connection_hops[0].clone(),
                }
                .into(),
            },
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateUpdateClient<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    identified!(LatestHeight<Tr, Hc>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData = HList![
        // identified!(LatestHeight<Tr, Hc>),
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateUpdateClient {
                    client_id,
                    __marker: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            t: IbcState {
                path: ClientStatePath {
                    client_id: trusted_client_state_client_id
                },
                height: _trusted_client_state_fetched_at_height,
                state: trusted_client_state
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_client_id, client_id);
        assert_eq!(this_chain_id, self_chain_id);

        aggregate(
            [fetch(id(
                counterparty_chain_id,
                FetchLatestHeight {
                    __marker: PhantomData,
                },
            ))],
            [],
            id(
                this_chain_id,
                AggregateUpdateClientFromHeight {
                    from_height: trusted_client_state.height(),
                    client_id,
                },
            ),
        )
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateUpdateClientFromHeight<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    identified!(LatestHeight<Tr, Hc>): IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData = HList![identified!(LatestHeight<Tr, Hc>),];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateUpdateClientFromHeight {
                    client_id,
                    from_height,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: counterparty_chain_id,
            t: LatestHeight {
                height: counterparty_latest_height,
                __marker
            },
            __marker: _,
        },]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        fetch(id::<Tr, Hc, _>(
            counterparty_chain_id,
            FetchUpdateHeaders {
                counterparty_client_id: client_id,
                counterparty_chain_id: this_chain_id,
                update_from: from_height,
                update_to: counterparty_latest_height,
            },
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateWaitForTrustedHeight<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Hc>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateWaitForTrustedHeight {
                    wait_for,
                    client_id,
                    counterparty_client_id,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: trusted_client_state_chain_id,
            t: IbcState {
                path: ClientStatePath {
                    client_id: trusted_client_state_client_id
                },
                height: _trusted_client_state_fetched_at_height,
                state: trusted_client_state
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(trusted_client_state_client_id, client_id);
        assert_eq!(trusted_client_state_chain_id, this_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        wait(id(
            counterparty_chain_id,
            WaitForTrustedHeight {
                height: wait_for,
                client_id: counterparty_client_id,
                counterparty_client_id: client_id,
                counterparty_chain_id: this_chain_id,
            },
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateMsgAfterUpdate<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t: msg_to_aggregate,
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: self_chain_id,
            t: IbcState {
                path: ClientStatePath {
                    client_id: trusted_client_state_client_id
                },
                height: trusted_client_state_fetched_at_height,
                state: trusted_client_state
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
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

                aggregate(
                    [
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ClientStatePath {
                                    client_id: event.client_id.clone(),
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ClientConsensusStatePath {
                                    client_id: event.client_id.clone(),
                                    height: trusted_client_state_height,
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }
                                .into(),
                            },
                        )),
                    ],
                    [id(
                        this_chain_id.clone(),
                        IbcState::<ClientStatePath<Hc::ClientId>, Hc, Tr> {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        },
                    )
                    .into()],
                    id(
                        this_chain_id,
                        AggregateConnectionOpenTry {
                            event_height,
                            event,
                        },
                    ),
                )
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

                aggregate(
                    [
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ClientStatePath {
                                    client_id: event.client_id.clone(),
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ClientConsensusStatePath {
                                    client_id: event.client_id.clone(),
                                    height: trusted_client_state_height,
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ConnectionPath {
                                    connection_id: event.connection_id.clone(),
                                }
                                .into(),
                            },
                        )),
                    ],
                    [id(
                        this_chain_id.clone(),
                        IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        },
                    )
                    .into()],
                    id(
                        this_chain_id,
                        AggregateConnectionOpenAck {
                            event_height,
                            event,
                        },
                    ),
                )
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

                aggregate(
                    [fetch(id::<Hc, Tr, _>(
                        this_chain_id.clone(),
                        FetchProof {
                            at: trusted_client_state_fetched_at_height,
                            path: ConnectionPath {
                                connection_id: event.connection_id.clone(),
                            }
                            .into(),
                        },
                    ))],
                    [id(
                        this_chain_id.clone(),
                        IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        },
                    )
                    .into()],
                    id(
                        this_chain_id,
                        AggregateConnectionOpenConfirm {
                            event_height,
                            event,
                        },
                    ),
                )
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

                aggregate(
                    [
                        aggregate(
                            [fetch(id::<Hc, Tr, _>(
                                this_chain_id.clone(),
                                FetchState {
                                    at: trusted_client_state_fetched_at_height,
                                    path: ChannelEndPath {
                                        port_id: event.port_id.clone(),
                                        channel_id: event.channel_id.clone(),
                                    }
                                    .into(),
                                },
                            ))],
                            [],
                            id(
                                this_chain_id.clone(),
                                AggregateConnectionFetchFromChannelEnd {
                                    at: trusted_client_state_fetched_at_height,
                                    __marker: PhantomData,
                                },
                            ),
                        ),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        )),
                    ],
                    [id(
                        this_chain_id.clone(),
                        IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        },
                    )
                    .into()],
                    id(
                        this_chain_id,
                        AggregateChannelOpenTry {
                            event_height,
                            event,
                            __marker: PhantomData,
                        },
                    ),
                )
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

                aggregate(
                    [
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        )),
                    ],
                    [id(
                        this_chain_id.clone(),
                        Data::ClientState(IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        }),
                    )
                    .into()],
                    id(
                        this_chain_id,
                        AggregateChannelOpenAck {
                            event_height,
                            event,
                            __marker: PhantomData,
                        },
                    ),
                )
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

                aggregate(
                    [
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchProof {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        )),
                        fetch(id::<Hc, Tr, _>(
                            this_chain_id.clone(),
                            FetchState {
                                at: trusted_client_state_fetched_at_height,
                                path: ChannelEndPath {
                                    port_id: event.port_id.clone(),
                                    channel_id: event.channel_id.clone(),
                                }
                                .into(),
                            },
                        )),
                    ],
                    [id(
                        this_chain_id.clone(),
                        IbcState {
                            path: ClientStatePath {
                                client_id: trusted_client_state_client_id,
                            },
                            height: trusted_client_state_fetched_at_height,
                            state: trusted_client_state,
                        },
                    )
                    .into()],
                    id(
                        this_chain_id,
                        AggregateChannelOpenConfirm {
                            event_height,
                            event,
                            __marker: PhantomData,
                        },
                    ),
                )
            }
            AggregateMsgAfterUpdate::RecvPacket(AggregateRecvPacket {
                event_height,
                event,
                __marker: _,
            }) => aggregate(
                [fetch(id::<Hc, Tr, _>(
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
                ))],
                [id(
                    this_chain_id.clone(),
                    IbcState {
                        path: ClientStatePath {
                            client_id: trusted_client_state_client_id,
                        },
                        height: trusted_client_state_fetched_at_height,
                        state: trusted_client_state,
                    },
                )
                .into()],
                id(
                    this_chain_id,
                    AggregateRecvPacket {
                        event_height,
                        event,
                        __marker: PhantomData,
                    },
                ),
            ),
            AggregateMsgAfterUpdate::AckPacket(AggregateAckPacket {
                event_height,
                event,
                tx_hash,
                counterparty_client_id,
            }) => aggregate(
                [
                    fetch(id::<Hc, Tr, _>(
                        this_chain_id.clone(),
                        FetchPacketAcknowledgement {
                            tx_hash,
                            destination_port_id: event.packet_dst_port.clone(),
                            destination_channel_id: event.packet_dst_channel.clone(),
                            sequence: event.packet_sequence,
                            __marker: PhantomData,
                        },
                    )),
                    fetch(id::<Hc, Tr, _>(
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
                    )),
                ],
                [id(
                    this_chain_id.clone(),
                    IbcState {
                        path: ClientStatePath {
                            client_id: trusted_client_state_client_id,
                        },
                        height: trusted_client_state_fetched_at_height,
                        state: trusted_client_state,
                    },
                )
                .into()],
                id(
                    this_chain_id,
                    AggregateAckPacket {
                        event_height,
                        event,
                        tx_hash,
                        counterparty_client_id,
                    },
                ),
            ),
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateConnectionOpenTry<Hc, Tr>)
where
    // state
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,

    // proof
    Identified<Hc, Tr, IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>: IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>,
        Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateConnectionOpenTry {
                    event_height: trusted_height,
                    event,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                t: IbcState {
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
                t: IbcProof {
                    height: client_state_proof_height,
                    proof: client_state_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: consensus_state_proof_chain_id,
                t: IbcProof {
                    height: consensus_state_proof_height,
                    proof: consensus_state_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: connection_proof_chain_id,
                t: IbcProof {
                    height: connection_proof_height,
                    proof: connection_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: connection_end_chain_id,
                t: IbcState {
                    path: _,
                    height: _,
                    state: connection_end
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert!(consensus_state_proof_height.revision_height() >= trusted_height.revision_height());
        assert!(client_state_proof_height.revision_height() >= trusted_height.revision_height());

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);

        assert_eq!(client_state_proof_chain_id, this_chain_id);
        assert_eq!(consensus_state_proof_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);
        assert_eq!(connection_end_chain_id, this_chain_id);

        let consensus_height = trusted_client_state.height();

        effect(id::<Tr, Hc, _>(
            counterparty_chain_id,
            MsgConnectionOpenTryData(MsgConnectionOpenTry {
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
            }),
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateConnectionOpenAck<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>,
        Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateConnectionOpenAck {
                    event_height: trusted_height,
                    event,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                t: IbcState {
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
                t: IbcProof {
                    height: client_state_proof_height,
                    proof: client_state_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: consensus_state_proof_chain_id,
                t: IbcProof {
                    height: consensus_state_proof_height,
                    proof: consensus_state_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: connection_proof_chain_id,
                t: IbcProof {
                    height: connection_proof_height,
                    proof: connection_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
            Identified {
                chain_id: connection_end_chain_id,
                t: IbcState {
                    path: _,
                    height: _,
                    state: connection_end
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert!(consensus_state_proof_height.revision_height() >= trusted_height.revision_height());
        assert!(client_state_proof_height.revision_height() >= trusted_height.revision_height());

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(client_state_proof_chain_id, this_chain_id);
        assert_eq!(consensus_state_proof_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);
        assert_eq!(connection_end_chain_id, this_chain_id);

        let consensus_height = trusted_client_state.height();

        effect(id::<Tr, Hc, _>(
            counterparty_chain_id,
            MsgConnectionOpenAckData(MsgConnectionOpenAck {
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
            }),
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateConnectionOpenConfirm<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateConnectionOpenConfirm {
                    event_height: _,
                    event,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                t: IbcState {
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
                t: IbcProof {
                    height: connection_proof_height,
                    proof: connection_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(connection_proof_chain_id, this_chain_id);

        effect(id::<Tr, Hc, _>(
            counterparty_chain_id,
            MsgConnectionOpenConfirmData {
                msg: MsgConnectionOpenConfirm {
                    connection_id: event.counterparty_connection_id,
                    proof_height: connection_proof_height,
                    proof_ack: connection_proof,
                },
                __marker: PhantomData,
            },
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateChannelOpenTry<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>,
        Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>,
        Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
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
                t: IbcState {
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
                t: IbcProof {
                    proof: channel_proof,
                    height: channel_proof_height,
                    path: _,
                    __marker: _,
                },
                __marker: _
            },
            Identified {
                chain_id: _connection_end_chain_id,
                t: IbcState {
                    path: _,
                    height: _,
                    state: connection,
                },

                __marker: _
            },
            Identified {
                chain_id: _channel_end_chain_id,
                t: IbcState {
                    path: _,
                    height: _,
                    state: channel,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(channel_proof_chain_id, this_chain_id);

        effect(id::<Tr, Hc, _>(
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
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateChannelOpenAck<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>,
        Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
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
                t: IbcState {
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
                t: IbcProof {
                    height: channel_proof_height,
                    proof: channel_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _
            },
            Identified {
                chain_id: channel_end_chain_id,
                t: IbcState {
                    path: _,
                    height: _,
                    state: channel,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        assert_eq!(trusted_client_state_chain_id, this_chain_id);
        assert_eq!(channel_proof_chain_id, this_chain_id);
        assert_eq!(channel_end_chain_id, this_chain_id);

        effect(id::<Tr, Hc, _>(
            counterparty_chain_id,
            MsgChannelOpenAckData {
                msg: MsgChannelOpenAck {
                    port_id: channel.counterparty.port_id.clone(),
                    channel_id: event.counterparty_channel_id,
                    counterparty_channel_id: event.channel_id,
                    counterparty_version: event.version,
                    proof_try: channel_proof,
                    proof_height: channel_proof_height,
                },
                __marker: PhantomData,
            },
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateChannelOpenConfirm<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>,
        Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
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
                t: IbcState {
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
                t: IbcProof {
                    height: channel_proof_height,
                    proof: channel_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _
            },
            Identified {
                chain_id: channel_end_chain_id,
                t: IbcState {
                    path: _,
                    height: _,
                    state: channel,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, channel_proof_chain_id);
        assert_eq!(channel_end_chain_id, this_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        effect(id::<Tr, Hc, _>(
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
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateRecvPacket<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<CommitmentPath, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        Identified<Hc, Tr, IbcProof<CommitmentPath, Hc, Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
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
                t: IbcState {
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
                t: IbcProof {
                    height: commitment_proof_height,
                    proof: commitment_proof,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, commitment_proof_chain_id);

        tracing::debug!("aggregate recv_packet");

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        effect(id::<Tr, Hc, _>(
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
                    proof_height: commitment_proof_height,
                },
                __marker: PhantomData,
            },
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateAckPacket<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<AcknowledgementPath, Hc, Tr>>: IsAggregateData,

    identified!(PacketAcknowledgement<Hc, Tr>): IsAggregateData,

    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,
{
    type AggregatedData = HList![
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>,
        identified!(PacketAcknowledgement<Hc, Tr>),
        Identified<Hc, Tr, IbcProof<AcknowledgementPath, Hc, Tr>>,
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateAckPacket {
                    event_height: _,
                    event,
                    tx_hash: _,
                    counterparty_client_id: _,
                },
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: trusted_client_state_chain_id,
                t: IbcState {
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
                t: PacketAcknowledgement { fetched_by: _, ack },
                __marker: _,
            },
            Identified {
                chain_id: commitment_proof_chain_id,
                t: IbcProof {
                    proof: acknowledgement_proof,
                    height: acknowledgement_proof_height,
                    path: _,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);
        assert_eq!(this_chain_id, packet_acknowledgement_chain_id);
        assert_eq!(commitment_proof_chain_id, this_chain_id);

        tracing::debug!("aggregate ack_packet");

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        effect(id::<Tr, Hc, _>(
            counterparty_chain_id,
            MsgAckPacketData {
                msg: MsgAcknowledgement {
                    proof_height: acknowledgement_proof_height,
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
        ))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateFetchCounterpartyStateProof<Hc, Tr>)
where
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t:
                AggregateFetchCounterpartyStateProof {
                    counterparty_client_id: _,
                    fetch: fetch_,
                },
            __marker: _,
        }: Self,
        hlist_pat![Identified {
            chain_id: trusted_client_state_chain_id,
            t: IbcState {
                height: _,
                path: _,
                state: trusted_client_state,
            },
            __marker: _,
        }]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(this_chain_id, trusted_client_state_chain_id);

        let counterparty_chain_id: ChainIdOf<Tr> = trusted_client_state.chain_id();

        fetch(id::<Tr, Hc, _>(counterparty_chain_id, fetch_.into()))
    }
}

impl<Hc: ChainExt, Tr: ChainExt> UseAggregate<RelayMessageTypes> for identified!(AggregateCreateClient<Hc, Tr>)
where
    identified!(SelfClientState<Tr, Hc>): IsAggregateData,
    identified!(SelfConsensusState<Tr, Hc>): IsAggregateData,
    AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Hc, Tr>)>,
{
    type AggregatedData = HList![
        identified!(SelfClientState<Tr, Hc>),
        identified!(SelfConsensusState<Tr, Hc>),
    ];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t: this,
            __marker: _,
        }: Self,
        hlist_pat![
            Identified {
                chain_id: self_client_state_chain_id,
                t: SelfClientState {
                    self_client_state,
                    __marker: _,
                },
                __marker: _
            },
            Identified {
                chain_id: self_consensus_state_chain_id,
                t: SelfConsensusState {
                    self_consensus_state,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> QueueMsg<RelayMessageTypes> {
        assert_eq!(self_client_state_chain_id, self_consensus_state_chain_id);

        effect(id::<Hc, Tr, _>(
            this_chain_id,
            MsgCreateClientData {
                config: this.config,
                msg: MsgCreateClient {
                    client_state: self_client_state,
                    consensus_state: self_consensus_state,
                },
            },
        ))
    }
}
