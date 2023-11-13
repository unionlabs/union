use std::fmt::Display;

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    hash::H256,
    traits::{Chain, ChainOf, HeightOf, LightClientBase},
    QueryHeight,
};

use crate::{
    aggregate,
    aggregate::{
        mk_aggregate_wait_for_update, Aggregate, AggregateChannelHandshakeUpdateClient,
        AggregateConnectionFetchFromChannelEnd, AggregateConnectionOpenAck,
        AggregateConnectionOpenConfirm, AggregateConnectionOpenTry, AggregateMsgAfterUpdate,
        AggregatePacketUpdateClient, AggregateUpdateClientFromClientId, AnyAggregate,
        ChannelHandshakeEvent, PacketEvent,
    },
    any_enum, fetch,
    fetch::{AnyFetch, Fetch, FetchChannelEnd, FetchConnectionEnd, FetchTrustedClientState},
    identified, seq, wait,
    wait::{AnyWait, Wait, WaitForBlock},
    AnyLightClientIdentified, LightClient, RelayerMsg,
};

any_enum! {
    #[any = AnyEvent]
    pub enum Event<L: LightClient> {
        Ibc(IbcEvent<L>),
        Command(Command<L>),
    }
}

impl<L: LightClient> Event<L> {
    pub fn handle(self, l: L) -> Vec<RelayerMsg>
    where
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<L>)>,
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<L>)>,
        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<L>)>,
    {
        match self {
            Event::Ibc(ibc_event) => match ibc_event.event {
                unionlabs::events::IbcEvent::CreateClient(e) => {
                    println!("client created: {e:#?}");

                    vec![]
                }
                unionlabs::events::IbcEvent::UpdateClient(e) => {
                    println!(
                        "client updated: {:#?} to {:#?}",
                        e.client_id, e.consensus_heights
                    );

                    vec![]
                }

                unionlabs::events::IbcEvent::ClientMisbehaviour(_) => unimplemented!(),
                unionlabs::events::IbcEvent::SubmitEvidence(_) => unimplemented!(),

                unionlabs::events::IbcEvent::ConnectionOpenInit(init) => [seq([
                    wait::<L>(l.chain().chain_id(), WaitForBlock(ibc_event.height)),
                    RelayerMsg::Aggregate {
                        data: [].into(),
                        queue: [mk_aggregate_wait_for_update(
                            l.chain().chain_id(),
                            init.client_id.clone(),
                            init.counterparty_client_id.clone(),
                            ibc_event.height,
                        )]
                        .into(),
                        receiver: aggregate(
                            l.chain().chain_id(),
                            AggregateMsgAfterUpdate::ConnectionOpenTry(
                                AggregateConnectionOpenTry {
                                    event_height: ibc_event.height,
                                    event: init,
                                },
                            ),
                        ),
                    },
                ])]
                .into(),
                unionlabs::events::IbcEvent::ConnectionOpenTry(try_) => {
                    [seq([RelayerMsg::Aggregate {
                        data: [].into(),
                        queue: [mk_aggregate_wait_for_update(
                            l.chain().chain_id(),
                            try_.client_id.clone(),
                            try_.counterparty_client_id.clone(),
                            ibc_event.height,
                        )]
                        .into(),
                        receiver: aggregate(
                            l.chain().chain_id(),
                            AggregateMsgAfterUpdate::ConnectionOpenAck(
                                AggregateConnectionOpenAck {
                                    event_height: ibc_event.height,
                                    event: try_,
                                },
                            ),
                        ),
                    }])]
                    .into()
                }
                unionlabs::events::IbcEvent::ConnectionOpenAck(ack) => {
                    [seq([RelayerMsg::Aggregate {
                        data: [].into(),
                        queue: [mk_aggregate_wait_for_update(
                            l.chain().chain_id(),
                            ack.client_id.clone(),
                            ack.counterparty_client_id.clone(),
                            ibc_event.height,
                        )]
                        .into(),
                        receiver: aggregate(
                            l.chain().chain_id(),
                            AggregateMsgAfterUpdate::ConnectionOpenConfirm(
                                AggregateConnectionOpenConfirm {
                                    event_height: ibc_event.height,
                                    event: ack,
                                },
                            ),
                        ),
                    }])]
                    .into()
                }
                unionlabs::events::IbcEvent::ConnectionOpenConfirm(confirm) => {
                    println!("connection opened: {confirm:#?}");

                    vec![]
                }

                unionlabs::events::IbcEvent::ChannelOpenInit(init) => {
                    [seq([RelayerMsg::Aggregate {
                        data: [].into(),
                        queue: [RelayerMsg::Aggregate {
                            data: [].into(),
                            queue: [fetch(
                                l.chain().chain_id(),
                                FetchChannelEnd {
                                    at: ibc_event.height,
                                    port_id: init.port_id.clone(),
                                    channel_id: init.channel_id.clone(),
                                },
                            )]
                            .into(),
                            receiver: aggregate(
                                l.chain().chain_id(),
                                AggregateConnectionFetchFromChannelEnd {
                                    at: ibc_event.height,
                                },
                            ),
                        }]
                        .into(),
                        receiver: aggregate(
                            l.chain().chain_id(),
                            AggregateChannelHandshakeUpdateClient {
                                update_to: ibc_event.height,
                                event_height: ibc_event.height,
                                channel_handshake_event: ChannelHandshakeEvent::Init(init),
                            },
                        ),
                    }])]
                    .into()
                }
                unionlabs::events::IbcEvent::ChannelOpenTry(try_) => {
                    [seq([RelayerMsg::Aggregate {
                        data: [].into(),
                        queue: [RelayerMsg::Aggregate {
                            data: [].into(),
                            queue: [fetch(
                                l.chain().chain_id(),
                                FetchChannelEnd {
                                    at: ibc_event.height,
                                    port_id: try_.port_id.clone(),
                                    channel_id: try_.channel_id.clone(),
                                },
                            )]
                            .into(),
                            receiver: aggregate(
                                l.chain().chain_id(),
                                Aggregate::ConnectionFetchFromChannelEnd(
                                    AggregateConnectionFetchFromChannelEnd {
                                        at: ibc_event.height,
                                    },
                                ),
                            ),
                        }]
                        .into(),
                        receiver: aggregate(
                            l.chain().chain_id(),
                            AggregateChannelHandshakeUpdateClient {
                                update_to: ibc_event.height,
                                event_height: ibc_event.height,
                                channel_handshake_event: ChannelHandshakeEvent::Try(try_),
                            },
                        ),
                    }])]
                    .into()
                }
                unionlabs::events::IbcEvent::ChannelOpenAck(ack) => {
                    [seq([RelayerMsg::Aggregate {
                        data: [].into(),
                        queue: [RelayerMsg::Aggregate {
                            data: [].into(),
                            queue: [fetch(
                                l.chain().chain_id(),
                                FetchChannelEnd {
                                    at: ibc_event.height,
                                    port_id: ack.port_id.clone(),
                                    channel_id: ack.channel_id.clone(),
                                },
                            )]
                            .into(),
                            receiver: aggregate(
                                l.chain().chain_id(),
                                AggregateConnectionFetchFromChannelEnd {
                                    at: ibc_event.height,
                                },
                            ),
                        }]
                        .into(),
                        receiver: aggregate(
                            l.chain().chain_id(),
                            AggregateChannelHandshakeUpdateClient {
                                update_to: ibc_event.height,
                                event_height: ibc_event.height,
                                channel_handshake_event: ChannelHandshakeEvent::Ack(ack),
                            },
                        ),
                    }])]
                    .into()
                }

                unionlabs::events::IbcEvent::ChannelOpenConfirm(confirm) => {
                    println!("channel opened: {confirm:#?}");

                    vec![]
                }

                unionlabs::events::IbcEvent::RecvPacket(packet) => [seq([RelayerMsg::Aggregate {
                    data: [].into(),
                    queue: [fetch(
                        l.chain().chain_id(),
                        FetchConnectionEnd {
                            at: ibc_event.height,
                            connection_id: packet.connection_id.clone(),
                        },
                    )]
                    .into(),
                    receiver: aggregate(
                        l.chain().chain_id(),
                        AggregatePacketUpdateClient {
                            update_to: ibc_event.height,
                            event_height: ibc_event.height,
                            block_hash: ibc_event.block_hash,
                            packet_event: PacketEvent::Recv(packet),
                        },
                    ),
                }])]
                .into(),
                unionlabs::events::IbcEvent::SendPacket(packet) => [seq([RelayerMsg::Aggregate {
                    data: [].into(),
                    queue: [fetch(
                        l.chain().chain_id(),
                        FetchConnectionEnd {
                            at: ibc_event.height,
                            connection_id: packet.connection_id.clone(),
                        },
                    )]
                    .into(),
                    receiver: aggregate(
                        l.chain().chain_id(),
                        AggregatePacketUpdateClient {
                            update_to: ibc_event.height,
                            event_height: ibc_event.height,
                            block_hash: ibc_event.block_hash,
                            packet_event: PacketEvent::Send(packet),
                        },
                    ),
                }])]
                .into(),
                unionlabs::events::IbcEvent::AcknowledgePacket(ack) => {
                    tracing::info!(?ack, "packet acknowledged");
                    [].into()
                }
                unionlabs::events::IbcEvent::TimeoutPacket(timeout) => {
                    tracing::error!(?timeout, "packet timed out");
                    [].into()
                }
                unionlabs::events::IbcEvent::WriteAcknowledgement(write_ack) => {
                    tracing::info!(?write_ack, "packet acknowledgement written");
                    [].into()
                }
            },
            Event::Command(command) => match command {
                Command::UpdateClient {
                    client_id,
                    counterparty_client_id,
                } => [RelayerMsg::Aggregate {
                    queue: [fetch::<L>(
                        l.chain().chain_id(),
                        FetchTrustedClientState {
                            at: QueryHeight::Latest,
                            client_id: client_id.clone(),
                        },
                    )]
                    .into(),
                    data: [].into(),
                    receiver: aggregate(
                        l.chain().chain_id(),
                        AggregateUpdateClientFromClientId {
                            client_id,
                            counterparty_client_id,
                        },
                    ),
                }]
                .into(),
            },
        }
    }
}

impl<L: LightClient> Display for Event<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Ibc(_) => write!(f, "Ibc"),
            Event::Command(cmd) => write!(f, "{cmd}"),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct IbcEvent<L: LightClient> {
    pub block_hash: H256,
    pub height: HeightOf<ChainOf<L>>,
    pub event: unionlabs::events::IbcEvent<
        L::ClientId,
        L::ClientType,
        <L::Counterparty as LightClientBase>::ClientId,
    >,
}

impl<L: LightClient> Display for IbcEvent<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use unionlabs::events::IbcEvent::*;

        match self.event {
            CreateClient(_) => write!(f, "Ibc::CreateClient"),
            UpdateClient(_) => write!(f, "Ibc::UpdateClient"),
            ClientMisbehaviour(_) => write!(f, "Ibc::ClientMisbehaviour"),
            SubmitEvidence(_) => write!(f, "Ibc::SubmitEvidence"),
            ConnectionOpenInit(_) => write!(f, "Ibc::ConnectionOpenInit"),
            ConnectionOpenTry(_) => write!(f, "Ibc::ConnectionOpenTry"),
            ConnectionOpenAck(_) => write!(f, "Ibc::ConnectionOpenAck"),
            ConnectionOpenConfirm(_) => write!(f, "Ibc::ConnectionOpenConfirm"),
            ChannelOpenInit(_) => write!(f, "Ibc::ChannelOpenInit"),
            ChannelOpenTry(_) => write!(f, "Ibc::ChannelOpenTry"),
            ChannelOpenAck(_) => write!(f, "Ibc::ChannelOpenAck"),
            ChannelOpenConfirm(_) => write!(f, "Ibc::ChannelOpenConfirm"),
            WriteAcknowledgement(_) => write!(f, "Ibc::WriteAcknowledgement"),
            RecvPacket(_) => write!(f, "Ibc::RecvPacket"),
            SendPacket(_) => write!(f, "Ibc::SendPacket"),
            AcknowledgePacket(_) => write!(f, "Ibc::AcknowledgePacket"),
            TimeoutPacket(_) => write!(f, "Ibc::TimeoutPacket"),
        }
    }
}

#[allow(non_camel_case_types, non_upper_case_globals)]
#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[display(fmt = "Command::{}")]
pub enum Command<L: LightClient> {
    #[display(fmt = "UpdateClient({client_id}, {counterparty_client_id})")]
    UpdateClient {
        client_id: L::ClientId,
        counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    },
}
