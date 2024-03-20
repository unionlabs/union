use std::{collections::VecDeque, fmt::Debug};

use chain_utils::{
    ethereum::IBCHandlerEvents,
    scroll::{Scroll, SCROLL_REVISION_NUMBER},
};
use contracts::{
    ibc_channel_handshake::{
        ChannelOpenAckFilter, ChannelOpenConfirmFilter, ChannelOpenInitFilter,
        ChannelOpenTryFilter, IBCChannelHandshakeEvents,
    },
    ibc_client::{ClientCreatedFilter, ClientUpdatedFilter, IBCClientEvents},
    ibc_connection::{
        ConnectionOpenAckFilter, ConnectionOpenConfirmFilter, ConnectionOpenInitFilter,
        ConnectionOpenTryFilter, IBCConnectionEvents,
    },
    ibc_packet::{AcknowledgePacketFilter, IBCPacketEvents, RecvPacketFilter, SendPacketFilter},
};
use enumorph::Enumorph;
use ethers::{contract::EthLogDecode, providers::Middleware, types::Filter};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use futures::StreamExt;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    conc, data, fetch, QueueMsg,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{DecodeAs, EthAbi},
    ethereum::config::Mainnet,
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
        CreateClient, IbcEvent, RecvPacket, SendPacket, UpdateClient,
    },
    hash::H256,
    ibc::{
        core::{
            channel::channel::Channel, client::height::Height,
            connection::connection_end::ConnectionEnd,
        },
        lightclients::cometbls,
    },
    proof::{ChannelEndPath, ConnectionPath},
    traits::{Chain, ChainIdOf, ClientIdOf, HeightOf},
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::ethereum::{
        FetchBeaconBlockRange, FetchChannel, FetchConnection, FetchEvents, FetchGetLogs,
    },
    data::{AnyData, ChainEvent, Data},
    fetch::{AnyFetch, DoFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockPollingTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
};

impl ChainExt for Scroll {
    type Data = ScrollData;
    type Fetch = ScrollFetch;
    type Aggregate = ScrollAggregate;
}

impl DoFetchBlockRange<Scroll> for Scroll
where
    AnyChainIdentified<AnyFetch>: From<Identified<Scroll, Fetch<Scroll>>>,
{
    fn fetch_block_range(
        c: &Scroll,
        range: FetchBlockRange<Scroll>,
    ) -> QueueMsg<BlockPollingTypes> {
        fetch(id(
            c.chain_id(),
            Fetch::<Scroll>::specific(FetchEvents {
                from_height: range.from_height,
                to_height: range.to_height,
            }),
        ))
    }
}

impl DoFetch<Scroll> for ScrollFetch
where
    AnyChainIdentified<AnyData>: From<Identified<Scroll, Data<Scroll>>>,
    AnyChainIdentified<AnyAggregate>: From<Identified<Scroll, Aggregate<Scroll>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Scroll, Fetch<Scroll>>>,
{
    async fn do_fetch(c: &Scroll, msg: Self) -> QueueMsg<BlockPollingTypes> {
        match msg {
            ScrollFetch::FetchEvents(FetchEvents {
                from_height,
                to_height,
            }) => fetch(id(
                c.chain_id(),
                Fetch::<Scroll>::specific(FetchBeaconBlockRange {
                    from_slot: from_height.revision_height,
                    to_slot: to_height.revision_height,
                }),
            )),
            ScrollFetch::FetchGetLogs(FetchGetLogs { from_slot, to_slot }) => {
                let event_height = Height {
                    revision_number: SCROLL_REVISION_NUMBER,
                    revision_height: to_slot,
                };

                let from_batch_index = c
                    .batch_index_of_beacon_height(Height {
                        revision_number: SCROLL_REVISION_NUMBER,
                        revision_height: from_slot,
                    })
                    .await;
                let to_batch_index = c.batch_index_of_beacon_height(event_height).await;

                tracing::debug!("slot range {from_slot}..{to_slot} is batch index range {from_batch_index}..{to_batch_index}");

                assert!(from_batch_index <= to_batch_index);

                if from_batch_index == to_batch_index {
                    QueueMsg::Noop
                } else {
                    assert!(from_batch_index + 1 == to_batch_index);

                    let from_scroll_height = c.scroll_height_of_batch_index(from_batch_index).await;
                    let to_scroll_height = c.scroll_height_of_batch_index(to_batch_index).await;

                    tracing::debug!("batch index {from_slot}..{to_slot} is batch index range {from_batch_index}..{to_batch_index}");

                    conc(
                        futures::stream::iter(
                            c.provider
                                .get_logs(
                                    &Filter::new()
                                        .address(c.readonly_ibc_handler.address())
                                        .from_block(from_scroll_height)
                                        // NOTE: This -1 is very important, else events will be double fetched
                                        .to_block(to_scroll_height - 1),
                                )
                                .await
                                .unwrap()
                                .into_iter(),
                        )
                        .then(|log| async {
                            let tx_hash = log
                                .transaction_hash
                                .expect("log should have transaction_hash")
                                .into();

                            tracing::debug!(?log, "raw log");

                            let event = IBCHandlerEvents::decode_log(&log.into())
                                .expect("failed to decode ibc handler event");

                            match event {
                                IBCHandlerEvents::PacketEvent(
                                    IBCPacketEvents::AcknowledgePacketFilter(raw_event),
                                ) => with_channel(
                                    c.chain_id(),
                                    raw_event.packet.source_port.clone(),
                                    raw_event.packet.source_channel.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ChannelEvent(
                                    IBCChannelHandshakeEvents::ChannelCloseConfirmFilter(_),
                                )
                                | IBCHandlerEvents::ChannelEvent(
                                    IBCChannelHandshakeEvents::ChannelCloseInitFilter(_),
                                ) => todo!(),
                                IBCHandlerEvents::ChannelEvent(
                                    IBCChannelHandshakeEvents::ChannelOpenAckFilter(raw_event),
                                ) => with_channel(
                                    c.chain_id(),
                                    raw_event.port_id.clone(),
                                    raw_event.channel_id.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ChannelEvent(
                                    IBCChannelHandshakeEvents::ChannelOpenConfirmFilter(raw_event),
                                ) => with_channel(
                                    c.chain_id(),
                                    raw_event.port_id.clone(),
                                    raw_event.channel_id.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ChannelEvent(
                                    IBCChannelHandshakeEvents::ChannelOpenInitFilter(raw_event),
                                ) => with_channel(
                                    c.chain_id(),
                                    raw_event.port_id.clone(),
                                    raw_event.channel_id.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ChannelEvent(
                                    IBCChannelHandshakeEvents::ChannelOpenTryFilter(raw_event),
                                ) => with_channel(
                                    c.chain_id(),
                                    raw_event.port_id.clone(),
                                    raw_event.channel_id.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ConnectionEvent(
                                    IBCConnectionEvents::ConnectionOpenAckFilter(raw_event),
                                ) => with_connection(
                                    c.chain_id(),
                                    raw_event.connection_id.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ConnectionEvent(
                                    IBCConnectionEvents::ConnectionOpenConfirmFilter(raw_event),
                                ) => with_connection(
                                    c.chain_id(),
                                    raw_event.connection_id.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ConnectionEvent(
                                    IBCConnectionEvents::ConnectionOpenInitFilter(raw_event),
                                ) => with_connection(
                                    c.chain_id(),
                                    raw_event.connection_id.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ConnectionEvent(
                                    IBCConnectionEvents::ConnectionOpenTryFilter(raw_event),
                                ) => with_connection(
                                    c.chain_id(),
                                    raw_event.connection_id.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::ClientEvent(
                                    IBCClientEvents::ClientCreatedFilter(ClientCreatedFilter(
                                        client_id,
                                    )),
                                ) => {
                                    let client_type = c
                                        .readonly_ibc_handler
                                        .client_types(client_id.clone())
                                        .await
                                        .unwrap();

                                    let (client_state, success) = c
                                        .readonly_ibc_handler
                                        .get_client_state(client_id.clone())
                                        .await
                                        .unwrap();

                                    assert!(success);

                                    let client_state =
                                        cometbls::client_state::ClientState::decode_as::<EthAbi>(
                                            &client_state,
                                        )
                                        .unwrap();

                                    data(id(
                                        c.chain_id(),
                                        ChainEvent::<Scroll> {
                                            client_type: unionlabs::ClientType::Cometbls,
                                            tx_hash,
                                            height: event_height,
                                            event: IbcEvent::CreateClient(CreateClient {
                                                client_id: client_id.parse().unwrap(),
                                                client_type,
                                                consensus_height: client_state.latest_height,
                                            }),
                                        },
                                    ))
                                }
                                IBCHandlerEvents::ClientEvent(
                                    IBCClientEvents::ClientRegisteredFilter(_),
                                ) => QueueMsg::Noop,
                                IBCHandlerEvents::ClientEvent(
                                    IBCClientEvents::ClientUpdatedFilter(ClientUpdatedFilter(
                                        client_id,
                                    )),
                                ) => {
                                    let client_type = c
                                        .readonly_ibc_handler
                                        .client_types(client_id.clone())
                                        .await
                                        .unwrap();

                                    let (client_state, success) = c
                                        .readonly_ibc_handler
                                        .get_client_state(client_id.clone())
                                        .block(c.scroll_height_of_batch_index(to_batch_index).await)
                                        .await
                                        .unwrap();

                                    assert!(success);

                                    let client_state =
                                        cometbls::client_state::ClientState::decode_as::<EthAbi>(
                                            &client_state,
                                        )
                                        .unwrap();

                                    data(id(
                                        c.chain_id(),
                                        ChainEvent::<Scroll> {
                                            client_type: unionlabs::ClientType::Cometbls,
                                            tx_hash,
                                            height: event_height,
                                            event: IbcEvent::UpdateClient(UpdateClient {
                                                client_id: client_id.parse().unwrap(),
                                                client_type,
                                                consensus_heights: vec![client_state.latest_height],
                                            }),
                                        },
                                    ))
                                }
                                IBCHandlerEvents::PacketEvent(
                                    IBCPacketEvents::RecvPacketFilter(raw_event),
                                ) => with_channel(
                                    c.chain_id(),
                                    raw_event.packet.destination_port.clone(),
                                    raw_event.packet.destination_channel.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::PacketEvent(
                                    IBCPacketEvents::SendPacketFilter(raw_event),
                                ) => with_channel(
                                    c.chain_id(),
                                    raw_event.source_port.clone(),
                                    raw_event.source_channel.clone(),
                                    event_height,
                                    tx_hash,
                                    raw_event,
                                ),
                                IBCHandlerEvents::PacketEvent(
                                    IBCPacketEvents::WriteAcknowledgementFilter(raw_event),
                                ) => {
                                    // TODO: Build write ack
                                    tracing::info!("write acknowledgement: {raw_event:?}");
                                    QueueMsg::Noop
                                }
                                IBCHandlerEvents::PacketEvent(
                                    IBCPacketEvents::TimeoutPacketFilter(_),
                                ) => {
                                    todo!()
                                }
                                IBCHandlerEvents::OwnableEvent(_) => QueueMsg::Noop,
                            }
                        })
                        .collect::<Vec<_>>()
                        .await,
                    )
                }
            }
            ScrollFetch::FetchBeaconBlockRange(FetchBeaconBlockRange { from_slot, to_slot }) => {
                assert!(from_slot < to_slot);

                if to_slot - from_slot == 1 {
                    fetch(id(
                        c.chain_id(),
                        Fetch::<Scroll>::specific(FetchGetLogs { from_slot, to_slot }),
                    ))
                } else {
                    // attempt to shrink from..to
                    // note that this is *exclusive* on the `to`
                    for slot in (from_slot + 1)..to_slot {
                        tracing::info!("querying slot {slot}");
                        match c
                            .l1
                            .beacon_api_client
                            .block(beacon_api::client::BlockId::Slot(slot))
                            .await
                        {
                            Err(beacon_api::errors::Error::NotFound(
                                beacon_api::errors::NotFoundError {
                                    message,
                                    error,
                                    status_code,
                                },
                            )) => {
                                tracing::info!(%message, %error, %status_code, "beacon block not found for slot {slot}");
                                continue;
                            }
                            Err(err) => {
                                panic!("error fetching beacon block for slot {slot}: {err}")
                            }
                            Ok(_) => {
                                return conc([
                                    fetch(id(
                                        c.chain_id(),
                                        Fetch::<Scroll>::specific(FetchGetLogs {
                                            from_slot,
                                            to_slot: slot,
                                        }),
                                    )),
                                    fetch(id(
                                        c.chain_id(),
                                        Fetch::<Scroll>::specific(FetchBeaconBlockRange {
                                            from_slot: slot,
                                            to_slot,
                                        }),
                                    )),
                                ])
                            }
                        }
                    }

                    // if the range is not shrinkable (i.e. all blocks between `from` and `to` are missing, but `from` and `to` both exist), fetch logs between `from` and `to`
                    fetch(id(
                        c.chain_id(),
                        Fetch::<Scroll>::specific(FetchGetLogs { from_slot, to_slot }),
                    ))
                }
            }
            ScrollFetch::FetchChannel(FetchChannel { .. }) => {
                // data(id(
                //     c.chain_id(),
                //     ChainSpecificData::<Scroll>(
                //         ChannelData(
                //             // TODO: This should read from scroll chain
                //             // c.ibc_state_read_at_execution_height(
                //             //     GetChannelCall {
                //             //         port_id: path.port_id.to_string(),
                //             //         channel_id: path.channel_id.to_string(),
                //             //     },
                //             //     c.execution_height(height).await,
                //             // )
                //             // .await
                //             // .unwrap()
                //             // .unwrap()
                //             // .try_into()
                //             // .unwrap(),
                //             todo!(),
                //         )
                //         .into(),
                //     ),
                // ));
                todo!()
            }
            ScrollFetch::FetchConnection(FetchConnection { .. }) => {
                // data(id(
                //     c.chain_id(),
                //     ChainSpecificData::<Scroll>(
                //         ConnectionData(
                //             // TODO: This should read from scroll chain
                //             // c.ibc_state_read_at_execution_height(
                //             //     GetConnectionCall {
                //             //         connection_id: path.connection_id.to_string(),
                //             //     },
                //             //     c.execution_height(height).await,
                //             // )
                //             // .await
                //             // .unwrap()
                //             // .unwrap()
                //             // .try_into()
                //             // .unwrap(),
                //             todo!(),
                //         )
                //         .into(),
                //     ),
                // ));
                todo!()
            }
        }
    }
}

fn with_channel<T>(
    chain_id: ChainIdOf<Scroll>,
    port_id: String,
    channel_id: String,
    event_height: HeightOf<Scroll>,
    tx_hash: H256,
    raw_event: T,
) -> QueueMsg<BlockPollingTypes>
where
    AggregateWithChannel: From<EventInfo<T>>,

    AnyChainIdentified<AnyAggregate>: From<Identified<Scroll, Aggregate<Scroll>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Scroll, Fetch<Scroll>>>,
{
    aggregate(
        [fetch(id(
            chain_id,
            Fetch::<Scroll>::specific(FetchChannel {
                height: event_height,
                path: ChannelEndPath {
                    port_id: port_id.parse().unwrap(),
                    channel_id: channel_id.parse().unwrap(),
                },
            }),
        ))],
        [],
        id(
            chain_id,
            Aggregate::<Scroll>::specific(AggregateWithChannel::from(EventInfo {
                height: event_height,
                tx_hash,
                raw_event,
            })),
        ),
    )
}

fn with_connection<T>(
    chain_id: ChainIdOf<Scroll>,
    connection_id: String,
    event_height: HeightOf<Scroll>,
    tx_hash: H256,
    raw_event: T,
) -> QueueMsg<BlockPollingTypes>
where
    AggregateWithConnection: From<EventInfo<T>>,

    AnyChainIdentified<AnyAggregate>: From<Identified<Scroll, Aggregate<Scroll>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Scroll, Fetch<Scroll>>>,
{
    aggregate(
        [fetch(id(
            chain_id,
            Fetch::<Scroll>::specific(FetchConnection {
                height: event_height,
                path: ConnectionPath {
                    connection_id: connection_id.parse().unwrap(),
                },
            }),
        ))],
        [],
        id(
            chain_id,
            Aggregate::<Scroll>::specific(AggregateWithConnection::from(EventInfo {
                height: event_height,
                tx_hash,
                raw_event,
            })),
        ),
    )
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    Enumorph,
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(deny_unknown_fields)]
pub enum ScrollFetch {
    #[display(fmt = "FetchEvents")]
    FetchEvents(FetchEvents<Mainnet>),
    #[display(fmt = "FetchGetLogs({}..{})", "_0.from_slot", "_0.to_slot")]
    FetchGetLogs(FetchGetLogs),
    #[display(fmt = "FetchBeaconBlockRange")]
    FetchBeaconBlockRange(FetchBeaconBlockRange),

    #[display(fmt = "FetchChannel")]
    FetchChannel(FetchChannel),
    #[display(fmt = "FetchConnection")]
    FetchConnection(FetchConnection),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(deny_unknown_fields)]
pub struct FetchBatchIndex {
    beacon_slot: u64,
    batch_index: u64,
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    enumorph::Enumorph,
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields
)]
pub enum ScrollAggregate {
    #[display(fmt = "AggregateWithChannel")]
    AggregateWithChannel(AggregateWithChannel),
    #[display(fmt = "AggregateWithChannel")]
    AggregateWithConnection(AggregateWithConnection),
}

impl DoAggregate for Identified<Scroll, ScrollAggregate>
where
    AnyChainIdentified<AnyData>: From<Identified<Scroll, ChainEvent<Scroll>>>,

    Identified<Scroll, ChannelData>: IsAggregateData,
    Identified<Scroll, ConnectionData>: IsAggregateData,
{
    fn do_aggregate(
        Identified { chain_id, t }: Self,
        data: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> QueueMsg<BlockPollingTypes> {
        match t {
            ScrollAggregate::AggregateWithChannel(msg) => do_aggregate(id(chain_id, msg), data),
            ScrollAggregate::AggregateWithConnection(msg) => do_aggregate(id(chain_id, msg), data),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, Enumorph)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields
)]
pub enum AggregateWithChannel {
    PacketAcknowledgement(EventInfo<AcknowledgePacketFilter>),
    SendPacket(EventInfo<SendPacketFilter>),
    RecvPacket(EventInfo<RecvPacketFilter>),
    ChannelOpenInit(EventInfo<ChannelOpenInitFilter>),
    ChannelOpenTry(EventInfo<ChannelOpenTryFilter>),
    ChannelOpenAck(EventInfo<ChannelOpenAckFilter>),
    ChannelOpenConfirm(EventInfo<ChannelOpenConfirmFilter>),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, Enumorph)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields
)]
pub enum AggregateWithConnection {
    ConnectionOpenInit(EventInfo<ConnectionOpenInitFilter>),
    ConnectionOpenTry(EventInfo<ConnectionOpenTryFilter>),
    ConnectionOpenAck(EventInfo<ConnectionOpenAckFilter>),
    ConnectionOpenConfirm(EventInfo<ConnectionOpenConfirmFilter>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct EventInfo<T> {
    height: Height,
    tx_hash: H256,
    raw_event: T,
}

impl UseAggregate<BlockPollingTypes> for Identified<Scroll, AggregateWithChannel>
where
    Identified<Scroll, ChannelData>: IsAggregateData,

    AnyChainIdentified<AnyData>: From<Identified<Scroll, ChainEvent<Scroll>>>,
{
    type AggregatedData = HList![Identified<Scroll, ChannelData>];

    fn aggregate(
        Identified { t: msg, chain_id }: Self,
        hlist_pat![Identified {
            chain_id: channel_data_chain_id,
            t: ChannelData(channel)
        }]: Self::AggregatedData,
    ) -> QueueMsg<BlockPollingTypes> {
        assert_eq!(chain_id, channel_data_chain_id);

        let event = match msg {
            AggregateWithChannel::PacketAcknowledgement(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent::<Scroll> {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::AcknowledgePacket(AcknowledgePacket {
                    packet_timeout_height: raw_event.packet.timeout_height.into(),
                    packet_timeout_timestamp: raw_event.packet.timeout_timestamp,
                    packet_sequence: raw_event.packet.sequence.try_into().unwrap(),
                    packet_src_port: raw_event.packet.source_port.parse().unwrap(),
                    packet_src_channel: raw_event.packet.source_channel.parse().unwrap(),
                    packet_dst_port: raw_event.packet.destination_port.parse().unwrap(),
                    packet_dst_channel: raw_event.packet.destination_channel.parse().unwrap(),
                    packet_channel_ordering: channel.ordering,
                    connection_id: channel.connection_hops[0].clone(),
                }),
            },
            AggregateWithChannel::SendPacket(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => {
                ChainEvent {
                    client_type: unionlabs::ClientType::Cometbls,
                    tx_hash,
                    height,
                    event: IbcEvent::SendPacket(SendPacket {
                        packet_data_hex: raw_event.data.to_vec(),
                        packet_timeout_height: raw_event.timeout_height.into(),
                        packet_timeout_timestamp: raw_event.timeout_timestamp,
                        packet_sequence: raw_event.sequence.try_into().unwrap(),
                        packet_src_port: raw_event.source_port.parse().unwrap(),
                        packet_src_channel: raw_event.source_channel.parse().unwrap(),
                        // REVIEW: Should we query the packet instead? Or is that the same info? Is it even possible to
                        // query packets from the Scroll?
                        packet_dst_port: channel.counterparty.port_id,
                        packet_dst_channel: channel.counterparty.channel_id.parse().unwrap(),
                        packet_channel_ordering: channel.ordering,
                        connection_id: channel.connection_hops[0].clone(),
                    }),
                }
            }
            AggregateWithChannel::RecvPacket(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::RecvPacket(RecvPacket {
                    packet_data_hex: raw_event.packet.data.to_vec(),
                    packet_timeout_height: raw_event.packet.timeout_height.into(),
                    packet_timeout_timestamp: raw_event.packet.timeout_timestamp,
                    packet_sequence: raw_event.packet.sequence.try_into().unwrap(),

                    packet_src_port: raw_event.packet.source_port.parse().unwrap(),
                    packet_src_channel: raw_event.packet.source_channel.parse().unwrap(),
                    packet_dst_port: raw_event.packet.destination_port.parse().unwrap(),
                    packet_dst_channel: raw_event.packet.destination_channel.parse().unwrap(),
                    packet_channel_ordering: channel.ordering,
                    connection_id: channel.connection_hops[0].clone(),
                }),
            },
            AggregateWithChannel::ChannelOpenAck(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::ChannelOpenAck(ChannelOpenAck {
                    port_id: raw_event.port_id.parse().unwrap(),
                    channel_id: raw_event.channel_id.parse().unwrap(),
                    counterparty_port_id: channel.counterparty.port_id,
                    counterparty_channel_id: channel.counterparty.channel_id.parse().unwrap(),
                    connection_id: channel.connection_hops[0].clone(),
                }),
            },
            AggregateWithChannel::ChannelOpenConfirm(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
                    port_id: raw_event.port_id.parse().unwrap(),
                    channel_id: raw_event.channel_id.parse().unwrap(),
                    counterparty_port_id: channel.counterparty.port_id,
                    counterparty_channel_id: channel.counterparty.channel_id.parse().unwrap(),
                    connection_id: channel.connection_hops[0].clone(),
                }),
            },
            AggregateWithChannel::ChannelOpenInit(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::ChannelOpenInit(ChannelOpenInit {
                    port_id: raw_event.port_id.parse().unwrap(),
                    channel_id: raw_event.channel_id.parse().unwrap(),
                    counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    version: channel.version,
                }),
            },
            AggregateWithChannel::ChannelOpenTry(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::ChannelOpenTry(ChannelOpenTry {
                    port_id: raw_event.port_id.parse().unwrap(),
                    channel_id: raw_event.channel_id.parse().unwrap(),
                    counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                    counterparty_channel_id: channel.counterparty.channel_id.parse().unwrap(),
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    version: raw_event.version,
                }),
            },
        };

        data(id(chain_id, event))
    }
}

impl UseAggregate<BlockPollingTypes> for Identified<Scroll, AggregateWithConnection>
where
    Identified<Scroll, ConnectionData>: IsAggregateData,

    AnyChainIdentified<AnyData>: From<Identified<Scroll, ChainEvent<Scroll>>>,
{
    type AggregatedData = HList![Identified<Scroll, ConnectionData>];

    fn aggregate(
        Identified { t: msg, chain_id }: Self,
        hlist_pat![Identified {
            chain_id: connection_data_chain_id,
            t: ConnectionData(connection)
        }]: Self::AggregatedData,
    ) -> QueueMsg<BlockPollingTypes> {
        assert_eq!(chain_id, connection_data_chain_id);

        let event = match msg {
            AggregateWithConnection::ConnectionOpenInit(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent::<Scroll> {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    client_id: connection.client_id,
                    counterparty_client_id: connection.counterparty.client_id.parse().unwrap(),
                }),
            },
            AggregateWithConnection::ConnectionOpenTry(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    client_id: connection.client_id,
                    counterparty_client_id: connection.counterparty.client_id,
                    counterparty_connection_id: connection
                        .counterparty
                        .connection_id
                        .parse()
                        .unwrap(),
                }),
            },
            AggregateWithConnection::ConnectionOpenAck(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    client_id: connection.client_id,
                    counterparty_client_id: connection.counterparty.client_id,
                    counterparty_connection_id: connection
                        .counterparty
                        .connection_id
                        .parse()
                        .unwrap(),
                }),
            },
            AggregateWithConnection::ConnectionOpenConfirm(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    client_id: connection.client_id,
                    counterparty_client_id: connection.counterparty.client_id,
                    counterparty_connection_id: connection
                        .counterparty
                        .connection_id
                        .parse()
                        .unwrap(),
                }),
            },
        };

        data(id(chain_id, event))
    }
}

#[derive(
    DebugNoBound,
    CloneNoBound,
    PartialEqNoBound,
    Serialize,
    Deserialize,
    derive_more::Display,
    Enumorph,
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(deny_unknown_fields)]
pub enum ScrollData {
    #[display(fmt = "Channel")]
    Channel(ChannelData),
    #[display(fmt = "Connection")]
    Connection(ConnectionData),
}

const _: () = {
    try_from_block_poll_msg! {
        chain = Scroll,
        generics = (),
        msgs = ScrollData(
            Channel(ChannelData),
            Connection(ConnectionData),
        ),
    }
};

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct ChannelData(pub Channel);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
// REVIEW: Use something other than string here?
pub struct ConnectionData(pub ConnectionEnd<ClientIdOf<Scroll>, String, String>);
