use std::{collections::VecDeque, fmt::Debug};

use chain_utils::evm::{Evm, IBCHandlerEvents, EVM_REVISION_NUMBER};
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
    ibc_handler::{GetChannelCall, GetConnectionCall},
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
    data, fetch, seq, QueueMsg,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::config::ChainSpec,
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
    TryFromEthAbi,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate, ChainSpecificAggregate},
    data::{AnyData, ChainEvent, ChainSpecificData, Data},
    fetch::{AnyFetch, ChainSpecificFetch, DoFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockPollingTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
};

impl<C: ChainSpec> ChainExt for Evm<C> {
    type Data = EvmData<C>;
    type Fetch = EvmFetch<C>;
    type Aggregate = EvmAggregate;
}

impl<C: ChainSpec> DoFetchBlockRange<Evm<C>> for Evm<C>
where
    AnyChainIdentified<AnyFetch>: From<Identified<Evm<C>, ChainSpecificFetch<Evm<C>>>>,
{
    fn fetch_block_range(
        c: &Evm<C>,
        range: FetchBlockRange<Evm<C>>,
    ) -> QueueMsg<BlockPollingTypes> {
        fetch(id(
            c.chain_id(),
            ChainSpecificFetch::<Evm<C>>(
                FetchEvents {
                    from_height: range.from_height,
                    to_height: range.to_height,
                }
                .into(),
            ),
        ))
    }
}

impl<C: ChainSpec> DoFetch<Evm<C>> for EvmFetch<C>
where
    AnyChainIdentified<AnyData>: From<Identified<Evm<C>, Data<Evm<C>>>>,
    AnyChainIdentified<AnyAggregate>: From<Identified<Evm<C>, Aggregate<Evm<C>>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Evm<C>, Fetch<Evm<C>>>>,
{
    async fn do_fetch(c: &Evm<C>, msg: Self) -> QueueMsg<BlockPollingTypes> {
        match msg {
            EvmFetch::FetchEvents(FetchEvents {
                from_height,
                to_height,
            }) => fetch(id(
                c.chain_id(),
                ChainSpecificFetch::<Evm<C>>(
                    FetchBeaconBlockRange {
                        from_slot: from_height.revision_height,
                        to_slot: to_height.revision_height,
                    }
                    .into(),
                ),
            )),
            EvmFetch::FetchGetLogs(FetchGetLogs { from_slot, to_slot }) => {
                let event_height = Height {
                    revision_number: EVM_REVISION_NUMBER,
                    revision_height: to_slot,
                };

                let from_block = c
                    .execution_height(Height {
                        revision_number: EVM_REVISION_NUMBER,
                        revision_height: from_slot,
                    })
                    .await;
                let to_block = c.execution_height(event_height).await;

                seq(futures::stream::iter(
                    c.provider
                        .get_logs(
                            &Filter::new()
                                .address(c.readonly_ibc_handler.address())
                                .from_block(from_block)
                                // NOTE: This -1 is very important, else events will be double fetched
                                .to_block(to_block - 1),
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
                        ) => with_channel::<C, _>(
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
                        IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientCreatedFilter(
                            ClientCreatedFilter(client_id),
                        )) => {
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

                            dbg!(hex::encode(&client_state));

                            let client_state =
                                cometbls::client_state::ClientState::try_from_eth_abi_bytes(
                                    &client_state,
                                )
                                .unwrap();

                            data(Identified::<Evm<C>, _>::new(
                                c.chain_id(),
                                ChainEvent {
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
                        IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientRegisteredFilter(
                            _,
                        )) => QueueMsg::Noop,
                        IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientUpdatedFilter(
                            ClientUpdatedFilter(client_id),
                        )) => {
                            let client_type = c
                                .readonly_ibc_handler
                                .client_types(client_id.clone())
                                .await
                                .unwrap();

                            let (client_state, success) = c
                                .readonly_ibc_handler
                                .get_client_state(client_id.clone())
                                .block(c.execution_height(event_height).await)
                                .await
                                .unwrap();

                            assert!(success);

                            dbg!(hex::encode(&client_state));

                            let client_state =
                                cometbls::client_state::ClientState::try_from_eth_abi_bytes(
                                    &client_state,
                                )
                                .unwrap();

                            data(Identified::<Evm<C>, _>::new(
                                c.chain_id(),
                                ChainEvent {
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
                        IBCHandlerEvents::PacketEvent(IBCPacketEvents::RecvPacketFilter(
                            raw_event,
                        )) => with_channel(
                            c.chain_id(),
                            raw_event.packet.destination_port.clone(),
                            raw_event.packet.destination_channel.clone(),
                            event_height,
                            tx_hash,
                            raw_event,
                        ),
                        IBCHandlerEvents::PacketEvent(IBCPacketEvents::SendPacketFilter(
                            raw_event,
                        )) => with_channel(
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
                            println!("{raw_event:?}");
                            QueueMsg::Noop
                        }
                        IBCHandlerEvents::PacketEvent(IBCPacketEvents::TimeoutPacketFilter(_)) => {
                            todo!()
                        }
                        IBCHandlerEvents::OwnableEvent(_) => QueueMsg::Noop,
                    }
                })
                .collect::<Vec<_>>()
                .await)
            }
            EvmFetch::FetchBeaconBlockRange(FetchBeaconBlockRange { from_slot, to_slot }) => {
                assert!(from_slot < to_slot);

                if to_slot - from_slot == 1 {
                    fetch(id(
                        c.chain_id(),
                        ChainSpecificFetch::<Evm<C>>(EvmFetch::from(FetchGetLogs {
                            from_slot,
                            to_slot,
                        })),
                    ))
                } else {
                    // attempt to shrink from..to
                    // note that this is *exclusive* on the `to`
                    for slot in (from_slot + 1)..to_slot {
                        tracing::info!("querying slot {slot}");
                        match c
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
                                return seq([
                                    fetch(id(
                                        c.chain_id(),
                                        ChainSpecificFetch::<Evm<C>>(EvmFetch::from(
                                            FetchGetLogs {
                                                from_slot,
                                                to_slot: slot,
                                            },
                                        )),
                                    )),
                                    fetch(id(
                                        c.chain_id(),
                                        ChainSpecificFetch::<Evm<C>>(EvmFetch::from(
                                            FetchBeaconBlockRange {
                                                from_slot: slot,
                                                to_slot,
                                            },
                                        )),
                                    )),
                                ])
                            }
                        }
                    }

                    // if the range is not shrinkable (i.e. all blocks between `from` and `to` are missing, but `from` and `to` both exist), fetch logs between `from` and `to`
                    fetch(id(
                        c.chain_id(),
                        ChainSpecificFetch::<Evm<C>>(EvmFetch::from(FetchGetLogs {
                            from_slot,
                            to_slot,
                        })),
                    ))
                }
            }
            EvmFetch::FetchChannel(FetchChannel { height, path }) => data(id(
                c.chain_id(),
                ChainSpecificData::<Evm<C>>(
                    ChannelData(
                        c.ibc_state_read_at_execution_height(
                            GetChannelCall {
                                port_id: path.port_id.to_string(),
                                channel_id: path.channel_id.to_string(),
                            },
                            c.execution_height(height).await,
                        )
                        .await
                        .unwrap()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                    )
                    .into(),
                ),
            )),
            EvmFetch::FetchConnection(FetchConnection { height, path }) => data(id(
                c.chain_id(),
                ChainSpecificData::<Evm<C>>(
                    ConnectionData(
                        c.ibc_state_read_at_execution_height(
                            GetConnectionCall {
                                connection_id: path.connection_id.to_string(),
                            },
                            c.execution_height(height).await,
                        )
                        .await
                        .unwrap()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                    )
                    .into(),
                ),
            )),
        }
    }
}

fn with_channel<C: ChainSpec, T>(
    chain_id: ChainIdOf<Evm<C>>,
    port_id: String,
    channel_id: String,
    event_height: HeightOf<Evm<C>>,
    tx_hash: H256,
    raw_event: T,
) -> QueueMsg<BlockPollingTypes>
where
    EventInfo<T>: Into<AggregateWithChannel>,

    AnyChainIdentified<AnyAggregate>: From<Identified<Evm<C>, Aggregate<Evm<C>>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Evm<C>, Fetch<Evm<C>>>>,
{
    aggregate(
        [fetch(id(
            chain_id,
            ChainSpecificFetch::<Evm<C>>(
                FetchChannel {
                    height: event_height,
                    path: ChannelEndPath {
                        port_id: port_id.parse().unwrap(),
                        channel_id: channel_id.parse().unwrap(),
                    },
                }
                .into(),
            ),
        ))],
        [],
        Identified::<Evm<C>, _>::new(
            chain_id,
            ChainSpecificAggregate(EvmAggregate::AggregateWithChannel(
                EventInfo {
                    height: event_height,
                    tx_hash,
                    raw_event,
                }
                .into(),
            )),
        ),
    )
}

fn with_connection<C, T>(
    chain_id: ChainIdOf<Evm<C>>,
    connection_id: String,
    event_height: HeightOf<Evm<C>>,
    tx_hash: H256,
    raw_event: T,
) -> QueueMsg<BlockPollingTypes>
where
    C: ChainSpec,
    EventInfo<T>: Into<AggregateWithConnection>,

    AnyChainIdentified<AnyAggregate>: From<Identified<Evm<C>, Aggregate<Evm<C>>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Evm<C>, Fetch<Evm<C>>>>,
{
    aggregate(
        [fetch(id(
            chain_id,
            ChainSpecificFetch::<Evm<C>>(
                FetchConnection {
                    height: event_height,
                    path: ConnectionPath {
                        connection_id: connection_id.parse().unwrap(),
                    },
                }
                .into(),
            ),
        ))],
        [],
        Identified::<Evm<C>, _>::new(
            chain_id,
            ChainSpecificAggregate(EvmAggregate::AggregateWithConnection(
                EventInfo {
                    height: event_height,
                    tx_hash,
                    raw_event,
                }
                .into(),
            )),
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
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub enum EvmFetch<C: ChainSpec> {
    #[display(fmt = "FetchEvents")]
    FetchEvents(FetchEvents<C>),
    #[display(fmt = "FetchGetLogs")]
    FetchGetLogs(FetchGetLogs),
    #[display(fmt = "FetchBeaconBlockRange")]
    FetchBeaconBlockRange(FetchBeaconBlockRange),

    #[display(fmt = "FetchChannel")]
    FetchChannel(FetchChannel),
    #[display(fmt = "FetchConnection")]
    FetchConnection(FetchConnection),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct FetchEvents<C: ChainSpec> {
    pub from_height: HeightOf<Evm<C>>,
    pub to_height: HeightOf<Evm<C>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(deny_unknown_fields)]
pub struct FetchGetLogs {
    pub from_slot: u64,
    pub to_slot: u64,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(deny_unknown_fields)]
/// NOTE: This isn't just fetching one block because sometimes beacon slots are missed. We need to be able to fetch a range of slots to account for this.
/// The range is `[from_slot..to_slot)`, so to fetch a single block `N`, the range would be `N..N+1`.
pub struct FetchBeaconBlockRange {
    pub from_slot: u64,
    pub to_slot: u64,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(deny_unknown_fields)]
pub struct FetchChannel {
    pub height: Height,
    pub path: ChannelEndPath,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(deny_unknown_fields)]
pub struct FetchConnection {
    pub height: Height,
    pub path: ConnectionPath,
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields
)]
pub enum EvmAggregate {
    #[display(fmt = "AggregateWithChannel")]
    AggregateWithChannel(AggregateWithChannel),
    #[display(fmt = "AggregateWithChannel")]
    AggregateWithConnection(AggregateWithConnection),
}

impl<C: ChainSpec> DoAggregate for Identified<Evm<C>, EvmAggregate>
where
    AnyChainIdentified<AnyData>: From<Identified<Evm<C>, ChainEvent<Evm<C>>>>,

    Identified<Evm<C>, ChannelData>: IsAggregateData,
    Identified<Evm<C>, ConnectionData<C>>: IsAggregateData,
{
    fn do_aggregate(
        Identified { chain_id, t }: Self,
        data: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> QueueMsg<BlockPollingTypes> {
        match t {
            EvmAggregate::AggregateWithChannel(msg) => {
                do_aggregate(Identified::<Evm<C>, _>::new(chain_id, msg), data)
            }
            EvmAggregate::AggregateWithConnection(msg) => {
                do_aggregate(Identified::<Evm<C>, _>::new(chain_id, msg), data)
            }
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

impl<C: ChainSpec> UseAggregate<BlockPollingTypes> for Identified<Evm<C>, AggregateWithChannel>
where
    Identified<Evm<C>, ChannelData>: IsAggregateData,

    AnyChainIdentified<AnyData>: From<Identified<Evm<C>, ChainEvent<Evm<C>>>>,
{
    type AggregatedData = HList![Identified<Evm<C>, ChannelData>];

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
            }) => ChainEvent {
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
                        // query packets from the evm?
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

        data(Identified::<Evm<C>, _>::new(chain_id, event))
    }
}

impl<C: ChainSpec> UseAggregate<BlockPollingTypes> for Identified<Evm<C>, AggregateWithConnection>
where
    Identified<Evm<C>, ConnectionData<C>>: IsAggregateData,

    AnyChainIdentified<AnyData>: From<Identified<Evm<C>, ChainEvent<Evm<C>>>>,
{
    type AggregatedData = HList![Identified<Evm<C>, ConnectionData<C>>];

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
            }) => ChainEvent {
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

        data(Identified::<Evm<C>, _>::new(chain_id, event))
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
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub enum EvmData<C: ChainSpec> {
    #[display(fmt = "Channel")]
    Channel(ChannelData),
    #[display(fmt = "Connection")]
    Connection(ConnectionData<C>),
}

const _: () = {
    try_from_block_poll_msg! {
        chain = Evm<C>,
        generics = (C: ChainSpec),
        msgs = EvmData(
            Channel(ChannelData),
            Connection(ConnectionData<C>),
        ),
    }
};

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(deny_unknown_fields)]
pub struct ChannelData(pub Channel);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainSpec")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
// REVIEW: Use something other than string here?
pub struct ConnectionData<C: ChainSpec>(pub ConnectionEnd<ClientIdOf<Evm<C>>, String, String>);
