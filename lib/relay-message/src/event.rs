use std::marker::PhantomData;

use chain_utils::GetChain;
use macros::apply;
use queue_msg::{
    aggregate, conc, fetch, noop, queue_msg, wait, HandleEvent, QueueError, QueueMessageTypes,
    QueueMsg,
};
use tracing::{info, instrument};
use unionlabs::{
    events::UpdateClient,
    hash::H256,
    ibc::core::channel::packet::Packet,
    ics24::{ChannelEndPath, ClientStatePath, ConnectionPath},
    traits::{ClientIdOf, ClientTypeOf, HeightOf},
    QueryHeight,
};

use crate::{
    aggregate::{
        mk_aggregate_wait_for_update, Aggregate, AggregateChannelHandshakeMsgAfterUpdate,
        AggregateClientStateFromConnection, AggregateConnectionFetchFromChannelEnd,
        AggregateMsgAfterUpdate, AggregateMsgConnectionOpenAck, AggregateMsgConnectionOpenConfirm,
        AggregateMsgConnectionOpenTry, AggregatePacketMsgAfterUpdate, AggregatePacketTimeout,
        AggregateUpdateClient, AnyAggregate, ChannelHandshakeEvent, PacketEvent,
    },
    any_enum, any_lc,
    fetch::{AnyFetch, Fetch, FetchState},
    id, identified, seq,
    wait::{AnyWait, Wait, WaitForHeight},
    AnyLightClientIdentified, ChainExt, RelayMessageTypes,
};

#[apply(any_enum)]
#[any = AnyEvent]
pub enum Event<Hc: ChainExt, Tr: ChainExt> {
    Ibc(IbcEvent<Hc, Tr>),
    Command(Command<Hc, Tr>),
}

impl HandleEvent<RelayMessageTypes> for AnyLightClientIdentified<AnyEvent> {
    #[instrument(skip_all, fields(chain_id = %self.chain_id()))]
    fn handle(
        self,
        store: &<RelayMessageTypes as QueueMessageTypes>::Store,
    ) -> Result<QueueMsg<RelayMessageTypes>, QueueError> {
        let wait = self;

        any_lc! {
            |wait| {
                store
                    .with_chain(&wait.chain_id, move |c| wait.t.handle(c))
                    .map_err(|e| QueueError::Fatal(Box::new(e)))
            }
        }
    }
}

impl<Hc: ChainExt, Tr: ChainExt> Event<Hc, Tr> {
    pub fn handle(self, hc: Hc) -> QueueMsg<RelayMessageTypes>
    where
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Hc, Tr>)>,
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Hc, Tr>)>,
    {
        match self {
            Event::Ibc(ibc_event) => {
                let event_name = ibc_event.event.name();

                match ibc_event.event {
                    unionlabs::events::IbcEvent::CreateClient(e) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            client_id = %e.client_id,
                            client_type = %e.client_type,
                            consensus_height = %e.consensus_height
                        );

                        noop()
                    }
                    unionlabs::events::IbcEvent::UpdateClient(UpdateClient {
                        client_id,
                        client_type,
                        consensus_heights,
                    }) => {
                        let consensus_heights = consensus_heights
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(",");

                        info!(event = %event_name, %client_id, %client_type, %consensus_heights);

                        noop()
                    }

                    unionlabs::events::IbcEvent::ClientMisbehaviour(_) => unimplemented!(),
                    unionlabs::events::IbcEvent::SubmitEvidence(_) => unimplemented!(),

                    unionlabs::events::IbcEvent::ConnectionOpenInit(init) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            connection_id = %init.connection_id,
                            client_id = %init.client_id,
                            counterparty_client_id = %init.counterparty_client_id
                        );

                        seq([
                            wait(id(
                                hc.chain_id(),
                                WaitForHeight {
                                    height: ibc_event.height,
                                    __marker: PhantomData,
                                },
                            )),
                            aggregate(
                                [mk_aggregate_wait_for_update(
                                    hc.chain_id(),
                                    init.client_id.clone(),
                                    init.counterparty_client_id.clone(),
                                    ibc_event.height,
                                )],
                                [],
                                id(
                                    hc.chain_id(),
                                    AggregateMsgAfterUpdate::ConnectionOpenTry(
                                        AggregateMsgConnectionOpenTry {
                                            event_height: ibc_event.height,
                                            event: init,
                                        },
                                    ),
                                ),
                            ),
                        ])
                    }
                    unionlabs::events::IbcEvent::ConnectionOpenTry(try_) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            connection_id = %try_.connection_id,
                            counterparty_connection_id = %try_.counterparty_connection_id,
                            client_id = %try_.client_id,
                            counterparty_client_id = %try_.counterparty_client_id
                        );

                        aggregate(
                            [mk_aggregate_wait_for_update(
                                hc.chain_id(),
                                try_.client_id.clone(),
                                try_.counterparty_client_id.clone(),
                                ibc_event.height,
                            )],
                            [],
                            id(
                                hc.chain_id(),
                                AggregateMsgAfterUpdate::ConnectionOpenAck(
                                    AggregateMsgConnectionOpenAck {
                                        event_height: ibc_event.height,
                                        event: try_,
                                    },
                                ),
                            ),
                        )
                    }
                    unionlabs::events::IbcEvent::ConnectionOpenAck(ack) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            connection_id = %ack.connection_id,
                            counterparty_connection_id = %ack.counterparty_connection_id,
                            client_id = %ack.client_id,
                            counterparty_client_id = %ack.counterparty_client_id
                        );

                        aggregate(
                            [mk_aggregate_wait_for_update(
                                hc.chain_id(),
                                ack.client_id.clone(),
                                ack.counterparty_client_id.clone(),
                                ibc_event.height,
                            )],
                            [],
                            id(
                                hc.chain_id(),
                                AggregateMsgAfterUpdate::ConnectionOpenConfirm(
                                    AggregateMsgConnectionOpenConfirm {
                                        event_height: ibc_event.height,
                                        event: ack,
                                    },
                                ),
                            ),
                        )
                    }
                    unionlabs::events::IbcEvent::ConnectionOpenConfirm(confirm) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            connection_id = %confirm.connection_id,
                            counterparty_connection_id = %confirm.counterparty_connection_id,
                            client_id = %confirm.client_id,
                            counterparty_client_id = %confirm.counterparty_client_id
                        );

                        noop()
                    }

                    unionlabs::events::IbcEvent::ChannelOpenInit(init) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            port_id = %init.port_id,
                            channel_id = %init.channel_id,
                            counterparty_port_id = %init.counterparty_port_id,
                            connection_id = %init.connection_id,
                            version = %init.version,
                        );

                        aggregate(
                            [aggregate(
                                [fetch(id::<Hc, Tr, _>(
                                    hc.chain_id(),
                                    FetchState {
                                        at: QueryHeight::Specific(ibc_event.height),
                                        path: ChannelEndPath {
                                            port_id: init.port_id.clone(),
                                            channel_id: init.channel_id.clone(),
                                        }
                                        .into(),
                                    },
                                ))],
                                [],
                                id(
                                    hc.chain_id(),
                                    AggregateConnectionFetchFromChannelEnd {
                                        at: ibc_event.height,
                                        __marker: PhantomData,
                                    },
                                ),
                            )],
                            [],
                            id(
                                hc.chain_id(),
                                AggregateChannelHandshakeMsgAfterUpdate {
                                    event_height: ibc_event.height,
                                    channel_handshake_event: ChannelHandshakeEvent::Init(init),
                                    __marker: PhantomData,
                                },
                            ),
                        )
                    }
                    unionlabs::events::IbcEvent::ChannelOpenTry(try_) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            port_id = %try_.port_id,
                            channel_id = %try_.channel_id,
                            counterparty_port_id = %try_.counterparty_port_id,
                            counterparty_channel_id = %try_.counterparty_channel_id,
                            connection_id = %try_.connection_id,
                            version = %try_.version,
                        );

                        aggregate(
                            [aggregate(
                                [fetch(id::<Hc, Tr, _>(
                                    hc.chain_id(),
                                    FetchState {
                                        at: QueryHeight::Specific(ibc_event.height),
                                        path: ChannelEndPath {
                                            port_id: try_.port_id.clone(),
                                            channel_id: try_.channel_id.clone(),
                                        }
                                        .into(),
                                    },
                                ))],
                                [],
                                id(
                                    hc.chain_id(),
                                    AggregateConnectionFetchFromChannelEnd {
                                        at: ibc_event.height,
                                        __marker: PhantomData,
                                    },
                                ),
                            )],
                            [],
                            id(
                                hc.chain_id(),
                                AggregateChannelHandshakeMsgAfterUpdate {
                                    event_height: ibc_event.height,
                                    channel_handshake_event: ChannelHandshakeEvent::Try(try_),
                                    __marker: PhantomData,
                                },
                            ),
                        )
                    }
                    unionlabs::events::IbcEvent::ChannelOpenAck(ack) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            port_id = %ack.port_id,
                            channel_id = %ack.channel_id,
                            counterparty_port_id = %ack.counterparty_port_id,
                            counterparty_channel_id = %ack.counterparty_channel_id,
                            connection_id = %ack.connection_id,
                        );

                        aggregate(
                            [aggregate(
                                [fetch(id::<Hc, Tr, _>(
                                    hc.chain_id(),
                                    FetchState {
                                        at: QueryHeight::Specific(ibc_event.height),
                                        path: ChannelEndPath {
                                            port_id: ack.port_id.clone(),
                                            channel_id: ack.channel_id.clone(),
                                        }
                                        .into(),
                                    },
                                ))],
                                [],
                                id(
                                    hc.chain_id(),
                                    AggregateConnectionFetchFromChannelEnd {
                                        at: ibc_event.height,
                                        __marker: PhantomData,
                                    },
                                ),
                            )],
                            [],
                            id(
                                hc.chain_id(),
                                AggregateChannelHandshakeMsgAfterUpdate {
                                    event_height: ibc_event.height,
                                    channel_handshake_event: ChannelHandshakeEvent::Ack(ack),
                                    __marker: PhantomData,
                                },
                            ),
                        )
                    }
                    unionlabs::events::IbcEvent::ChannelOpenConfirm(confirm) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            port_id = %confirm.port_id,
                            channel_id = %confirm.channel_id,
                            counterparty_port_id = %confirm.counterparty_port_id,
                            counterparty_channel_id = %confirm.counterparty_channel_id,
                            connection_id = %confirm.connection_id,
                        );

                        noop()
                    }
                    unionlabs::events::IbcEvent::SendPacket(send) => {
                        info!(
                            event = %event_name,
                            timeout_height = %send.packet_timeout_height,
                            timeout_timestamp = %send.packet_timeout_timestamp,
                            sequence = %send.packet_sequence,
                            src_port = %send.packet_src_port,
                            src_channel = %send.packet_src_channel,
                            dst_port = %send.packet_dst_port,
                            dst_channel = %send.packet_dst_channel,
                            channel_ordering = %send.packet_channel_ordering,
                            connection_id = %send.connection_id,
                        );

                        // in parallel, run height timeout, timestamp timeout, and send packet
                        conc([
                            aggregate(
                                [
                                    fetch(id(
                                        hc.chain_id(),
                                        FetchState::<Hc, Tr> {
                                            at: QueryHeight::Specific(ibc_event.height),
                                            path: ConnectionPath {
                                                connection_id: send.connection_id.clone(),
                                            }
                                            .into(),
                                        },
                                    )),
                                    aggregate(
                                        [fetch(id(
                                            hc.chain_id(),
                                            FetchState::<Hc, Tr> {
                                                at: QueryHeight::Specific(ibc_event.height),
                                                path: ConnectionPath {
                                                    connection_id: send.connection_id.clone(),
                                                }
                                                .into(),
                                            },
                                        ))],
                                        [],
                                        id(
                                            hc.chain_id(),
                                            AggregateClientStateFromConnection::<Hc, Tr> {
                                                at: ibc_event.height,
                                                __marker: PhantomData,
                                            },
                                        ),
                                    ),
                                ],
                                [],
                                id(
                                    hc.chain_id(),
                                    AggregatePacketTimeout {
                                        packet: Packet {
                                            sequence: send.packet_sequence,
                                            source_port: send.packet_src_port.clone(),
                                            source_channel: send.packet_src_channel.clone(),
                                            destination_port: send.packet_dst_port.clone(),
                                            destination_channel: send.packet_dst_channel.clone(),
                                            data: send.packet_data_hex.clone(),
                                            timeout_height: send.packet_timeout_height,
                                            timeout_timestamp: send.packet_timeout_timestamp,
                                        },
                                        __marker: PhantomData,
                                    },
                                ),
                            ),
                            aggregate(
                                [fetch(id::<Hc, Tr, _>(
                                    hc.chain_id(),
                                    FetchState {
                                        at: QueryHeight::Specific(ibc_event.height),
                                        path: ConnectionPath {
                                            connection_id: send.connection_id.clone(),
                                        }
                                        .into(),
                                    },
                                ))],
                                [],
                                id(
                                    hc.chain_id(),
                                    AggregatePacketMsgAfterUpdate {
                                        update_to: ibc_event.height,
                                        event_height: ibc_event.height,
                                        tx_hash: ibc_event.tx_hash,
                                        packet_event: PacketEvent::Send(send),
                                        __marker: PhantomData,
                                    },
                                ),
                            ),
                        ])
                    }
                    unionlabs::events::IbcEvent::RecvPacket(recv) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            timeout_height = %recv.packet_timeout_height,
                            timeout_timestamp = %recv.packet_timeout_timestamp,
                            sequence = %recv.packet_sequence,
                            src_port = %recv.packet_src_port,
                            src_channel = %recv.packet_src_channel,
                            dst_port = %recv.packet_dst_port,
                            dst_channel = %recv.packet_dst_channel,
                            channel_ordering = %recv.packet_channel_ordering,
                            connection_id = %recv.connection_id,
                        );

                        noop()
                    }
                    unionlabs::events::IbcEvent::AcknowledgePacket(ack) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            timeout_height = %ack.packet_timeout_height,
                            timeout_timestamp = %ack.packet_timeout_timestamp,
                            sequence = %ack.packet_sequence,
                            src_port = %ack.packet_src_port,
                            src_channel = %ack.packet_src_channel,
                            dst_port = %ack.packet_dst_port,
                            dst_channel = %ack.packet_dst_channel,
                            channel_ordering = %ack.packet_channel_ordering,
                            connection_id = %ack.connection_id,
                        );

                        noop()
                    }
                    unionlabs::events::IbcEvent::TimeoutPacket(timeout) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            timeout_height = %timeout.packet_timeout_height,
                            timeout_timestamp = %timeout.packet_timeout_timestamp,
                            sequence = %timeout.packet_sequence,
                            src_port = %timeout.packet_src_port,
                            src_channel = %timeout.packet_src_channel,
                            dst_port = %timeout.packet_dst_port,
                            dst_channel = %timeout.packet_dst_channel,
                            channel_ordering = %timeout.packet_channel_ordering,
                            connection_id = %timeout.connection_id,
                        );

                        noop()
                    }
                    unionlabs::events::IbcEvent::WriteAcknowledgement(write_ack) => {
                        info!(
                            height = %ibc_event.height,
                            event = %event_name,
                            timeout_height = %write_ack.packet_timeout_height,
                            timeout_timestamp = %write_ack.packet_timeout_timestamp,
                            sequence = %write_ack.packet_sequence,
                            src_port = %write_ack.packet_src_port,
                            src_channel = %write_ack.packet_src_channel,
                            dst_port = %write_ack.packet_dst_port,
                            dst_channel = %write_ack.packet_dst_channel,
                            ack = %::serde_utils::to_hex(&write_ack.packet_ack_hex),
                            connection_id = %write_ack.connection_id,
                        );

                        aggregate(
                            [fetch(id::<Hc, Tr, _>(
                                hc.chain_id(),
                                FetchState {
                                    at: QueryHeight::Specific(ibc_event.height),
                                    path: ConnectionPath {
                                        connection_id: write_ack.connection_id.clone(),
                                    }
                                    .into(),
                                },
                            ))],
                            [],
                            id(
                                hc.chain_id(),
                                AggregatePacketMsgAfterUpdate {
                                    update_to: ibc_event.height,
                                    event_height: ibc_event.height,
                                    tx_hash: ibc_event.tx_hash,
                                    packet_event: PacketEvent::WriteAck(write_ack),
                                    __marker: PhantomData,
                                },
                            ),
                        )
                    }
                }
            }
            Event::Command(command) => match command {
                Command::UpdateClient {
                    client_id,
                    __marker: _,
                } => aggregate(
                    [fetch(id::<Hc, Tr, _>(
                        hc.chain_id(),
                        FetchState {
                            at: QueryHeight::Latest,
                            path: ClientStatePath {
                                client_id: client_id.clone(),
                            }
                            .into(),
                        },
                    ))],
                    [],
                    id(
                        hc.chain_id(),
                        AggregateUpdateClient {
                            client_id,
                            __marker: PhantomData,
                        },
                    ),
                ),
            },
        }
    }
}

#[queue_msg]
pub struct IbcEvent<Hc: ChainExt, Tr: ChainExt> {
    pub tx_hash: H256,
    pub height: HeightOf<Hc>,
    pub event: unionlabs::events::IbcEvent<ClientIdOf<Hc>, ClientTypeOf<Hc>, ClientIdOf<Tr>>,
}

#[queue_msg]
pub enum Command<Hc: ChainExt, Tr: ChainExt> {
    UpdateClient {
        client_id: ClientIdOf<Hc>,
        #[serde(skip)]
        __marker: PhantomData<fn() -> Tr>,
    },
}
