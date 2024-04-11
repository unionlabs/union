use std::{collections::VecDeque, marker::PhantomData};

use beacon_api::client::BeaconApiClient;
use chain_utils::ethereum::{
    Ethereum, EthereumChain, EthereumChainExt as _, IBCHandlerEvents, IbcHandlerExt,
    ETHEREUM_REVISION_NUMBER,
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
    ibc_handler::{GetChannelCall, GetConnectionCall},
    ibc_packet::{AcknowledgePacketFilter, IBCPacketEvents, RecvPacketFilter, SendPacketFilter},
};
use ethers::{contract::EthLogDecode, providers::Middleware, types::Filter};
use frunk::{hlist_pat, HList};
use futures::StreamExt;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    conc, data, fetch, queue_msg, QueueMsg,
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    encoding::{DecodeAs, EthAbi},
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
    ics24::{ChannelEndPath, ConnectionPath},
    traits::{Chain, ChainIdOf, ClientIdOf, HeightOf},
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    data::{AnyData, ChainEvent, Data},
    fetch::{AnyFetch, DoFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockMessageTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
};

pub trait EthereumChainExt = ChainExt + EthereumChain;

impl<C: ChainSpec> ChainExt for Ethereum<C> {
    type Data = EthereumData<C>;
    type Fetch = EthereumFetch<C>;
    type Aggregate = EthereumAggregate<C>;
}

impl<C: ChainSpec> DoFetchBlockRange<Ethereum<C>> for Ethereum<C>
where
    AnyChainIdentified<AnyFetch>: From<Identified<Ethereum<C>, Fetch<Ethereum<C>>>>,
{
    fn fetch_block_range(
        c: &Ethereum<C>,
        range: FetchBlockRange<Ethereum<C>>,
    ) -> QueueMsg<BlockMessageTypes> {
        fetch(id(
            c.chain_id(),
            Fetch::<Ethereum<C>>::specific(FetchEvents {
                from_height: range.from_height,
                to_height: range.to_height,
            }),
        ))
    }
}

impl<C: ChainSpec> DoFetch<Ethereum<C>> for EthereumFetch<C>
where
    AnyChainIdentified<AnyData>: From<Identified<Ethereum<C>, Data<Ethereum<C>>>>,
    AnyChainIdentified<AnyAggregate>: From<Identified<Ethereum<C>, Aggregate<Ethereum<C>>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Ethereum<C>, Fetch<Ethereum<C>>>>,
{
    async fn do_fetch(c: &Ethereum<C>, msg: Self) -> QueueMsg<BlockMessageTypes> {
        match msg {
            EthereumFetch::FetchEvents(FetchEvents {
                from_height,
                to_height,
            }) => fetch(id(
                c.chain_id(),
                Fetch::<Ethereum<C>>::specific(FetchBeaconBlockRange {
                    from_slot: from_height.revision_height,
                    to_slot: to_height.revision_height,
                }),
            )),
            EthereumFetch::FetchGetLogs(FetchGetLogs { from_slot, to_slot }) => {
                let event_height = Height {
                    revision_number: ETHEREUM_REVISION_NUMBER,
                    revision_height: to_slot,
                };

                let from_block = c
                    .beacon_api_client
                    .execution_height(beacon_api::client::BlockId::Slot(from_slot))
                    .await
                    .unwrap();
                let to_block = c
                    .beacon_api_client
                    .execution_height(beacon_api::client::BlockId::Slot(to_slot))
                    .await
                    .unwrap();

                // REVIEW: Surely transactions and events can be fetched in parallel?
                conc(
                    futures::stream::iter(
                        c.provider
                            .get_logs(
                                &Filter::new()
                                    .address(ethers::types::H160::from(c.ibc_handler_address))
                                    .from_block(from_block)
                                    // NOTE: This -1 is very important, else events will be double fetched
                                    .to_block(to_block - 1),
                            )
                            .await
                            .unwrap(),
                    )
                    .then(|log| async {
                        let tx_hash = log
                            .transaction_hash
                            .expect("log should have transaction_hash")
                            .into();

                        tracing::debug!(?log, "raw log");

                        match IBCHandlerEvents::decode_log(&log.into()) {
                            Ok(event) => {
                                Some(mk_aggregate_event(c, event, event_height, tx_hash).await)
                            }
                            Err(e) => {
                                tracing::warn!("could not decode evm event {}", e);
                                None
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
                )
            }
            EthereumFetch::FetchBeaconBlockRange(beacon_block_range) => {
                fetch_beacon_block_range(c, beacon_block_range, &c.beacon_api_client).await
            }
            EthereumFetch::FetchChannel(FetchChannel { height, path }) => data(id(
                c.chain_id(),
                Data::<Ethereum<C>>::specific(ChannelData {
                    channel: c
                        .ibc_handler()
                        .eth_call(
                            GetChannelCall {
                                port_id: path.port_id.to_string(),
                                channel_id: path.channel_id.to_string(),
                            },
                            c.beacon_api_client
                                .execution_height(beacon_api::client::BlockId::Slot(
                                    height.revision_height,
                                ))
                                .await
                                .unwrap(),
                        )
                        .await
                        .unwrap()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                    __marker: PhantomData,
                }),
            )),
            EthereumFetch::FetchConnection(FetchConnection { height, path }) => data(id(
                c.chain_id(),
                Data::<Ethereum<C>>::specific(ConnectionData(
                    c.ibc_handler()
                        .eth_call(
                            GetConnectionCall {
                                connection_id: path.connection_id.to_string(),
                            },
                            c.beacon_api_client
                                .execution_height(beacon_api::client::BlockId::Slot(
                                    height.revision_height,
                                ))
                                .await
                                .unwrap(),
                        )
                        .await
                        .unwrap()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                )),
            )),
        }
    }
}

pub async fn fetch_beacon_block_range<C, Hc>(
    c: &Hc,
    FetchBeaconBlockRange { from_slot, to_slot }: FetchBeaconBlockRange,
    beacon_api_client: &BeaconApiClient<C>,
) -> QueueMsg<BlockMessageTypes>
where
    C: ChainSpec,
    Hc: ChainExt + EthereumChain,

    AnyChainIdentified<AnyFetch>: From<Identified<Hc, Fetch<Hc>>>,

    // TODO: Replace with associated type bounds
    Hc::Fetch: From<FetchGetLogs> + From<FetchBeaconBlockRange>,
{
    assert!(from_slot < to_slot);

    if to_slot - from_slot == 1 {
        fetch(id(
            c.chain_id(),
            Fetch::<Hc>::specific(FetchGetLogs { from_slot, to_slot }),
        ))
    } else {
        // attempt to shrink from..to
        // note that this is *exclusive* on the `to`
        for slot in (from_slot + 1)..to_slot {
            tracing::info!("querying slot {slot}");
            match beacon_api_client
                .block(beacon_api::client::BlockId::Slot(slot))
                .await
            {
                Err(beacon_api::errors::Error::NotFound(beacon_api::errors::NotFoundError {
                    message,
                    error,
                    status_code,
                })) => {
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
                            Fetch::<Hc>::specific(FetchGetLogs {
                                from_slot,
                                to_slot: slot,
                            }),
                        )),
                        fetch(id(
                            c.chain_id(),
                            Fetch::<Hc>::specific(FetchBeaconBlockRange {
                                from_slot: slot,
                                to_slot,
                            }),
                        )),
                    ]);
                }
            }
        }

        // if the range is not shrinkable (i.e. all blocks between `from` and `to` are missing, but `from` and `to` both exist), fetch logs between `from` and `to`
        fetch(id(
            c.chain_id(),
            Fetch::<Hc>::specific(FetchGetLogs { from_slot, to_slot }),
        ))
    }
}

pub async fn mk_aggregate_event<Hc>(
    c: &Hc,
    event: IBCHandlerEvents,
    event_height: Hc::Height,
    tx_hash: H256,
) -> QueueMsg<BlockMessageTypes>
where
    Hc: ChainExt + EthereumChain,
    Hc::Aggregate: From<AggregateWithChannel<Hc>>,
    Hc::Aggregate: From<AggregateWithConnection<Hc>>,
    Hc::Fetch: From<FetchChannel<Hc>>,
    Hc::Fetch: From<FetchConnection<Hc>>,

    AnyChainIdentified<AnyAggregate>: From<Identified<Hc, Aggregate<Hc>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Hc, Fetch<Hc>>>,
    AnyChainIdentified<AnyData>: From<Identified<Hc, Data<Hc>>>,
{
    match event {
        IBCHandlerEvents::PacketEvent(IBCPacketEvents::AcknowledgePacketFilter(raw_event)) => {
            with_channel::<Hc, _>(
                c.chain_id(),
                raw_event.packet.source_port.clone(),
                raw_event.packet.source_channel.clone(),
                event_height,
                tx_hash,
                raw_event,
            )
        }
        IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelCloseConfirmFilter(_))
        | IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelCloseInitFilter(_)) => {
            todo!()
        }
        IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenAckFilter(
            raw_event,
        )) => with_channel(
            c.chain_id(),
            raw_event.port_id.clone(),
            raw_event.channel_id.clone(),
            event_height,
            tx_hash,
            raw_event,
        ),
        IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenConfirmFilter(
            raw_event,
        )) => with_channel(
            c.chain_id(),
            raw_event.port_id.clone(),
            raw_event.channel_id.clone(),
            event_height,
            tx_hash,
            raw_event,
        ),
        IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenInitFilter(
            raw_event,
        )) => with_channel(
            c.chain_id(),
            raw_event.port_id.clone(),
            raw_event.channel_id.clone(),
            event_height,
            tx_hash,
            raw_event,
        ),
        IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenTryFilter(
            raw_event,
        )) => with_channel(
            c.chain_id(),
            raw_event.port_id.clone(),
            raw_event.channel_id.clone(),
            event_height,
            tx_hash,
            raw_event,
        ),
        IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenAckFilter(
            raw_event,
        )) => with_connection(
            c.chain_id(),
            raw_event.connection_id.clone(),
            event_height,
            tx_hash,
            raw_event,
        ),
        IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenConfirmFilter(
            raw_event,
        )) => with_connection(
            c.chain_id(),
            raw_event.connection_id.clone(),
            event_height,
            tx_hash,
            raw_event,
        ),
        IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenInitFilter(
            raw_event,
        )) => with_connection(
            c.chain_id(),
            raw_event.connection_id.clone(),
            event_height,
            tx_hash,
            raw_event,
        ),
        IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenTryFilter(
            raw_event,
        )) => with_connection(
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
                .ibc_handler()
                .client_types(client_id.clone())
                .await
                .unwrap();

            let (client_state, success) = c
                .ibc_handler()
                .get_client_state(client_id.clone())
                .await
                .unwrap();

            assert!(success);

            let client_state =
                cometbls::client_state::ClientState::decode_as::<EthAbi>(&client_state).unwrap();

            data(id(
                c.chain_id(),
                ChainEvent::<Hc> {
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
        IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientRegisteredFilter(_)) => QueueMsg::Noop,
        IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientUpdatedFilter(
            ClientUpdatedFilter(client_id, consensus_height),
        )) => {
            let client_type = c
                .ibc_handler()
                .client_types(client_id.clone())
                .await
                .unwrap();

            data(id(
                c.chain_id(),
                ChainEvent::<Hc> {
                    client_type: unionlabs::ClientType::Cometbls,
                    tx_hash,
                    height: event_height,
                    event: IbcEvent::UpdateClient(UpdateClient {
                        client_id: client_id.parse().unwrap(),
                        client_type,
                        consensus_heights: vec![consensus_height.into()],
                    }),
                },
            ))
        }
        IBCHandlerEvents::PacketEvent(IBCPacketEvents::RecvPacketFilter(raw_event)) => {
            with_channel(
                c.chain_id(),
                raw_event.packet.destination_port.clone(),
                raw_event.packet.destination_channel.clone(),
                event_height,
                tx_hash,
                raw_event,
            )
        }
        IBCHandlerEvents::PacketEvent(IBCPacketEvents::SendPacketFilter(raw_event)) => {
            with_channel(
                c.chain_id(),
                raw_event.source_port.clone(),
                raw_event.source_channel.clone(),
                event_height,
                tx_hash,
                raw_event,
            )
        }
        IBCHandlerEvents::PacketEvent(IBCPacketEvents::WriteAcknowledgementFilter(raw_event)) => {
            // TODO: Build write ack
            tracing::info!("write acknowledgement: {raw_event:?}");
            QueueMsg::Noop
        }
        IBCHandlerEvents::PacketEvent(IBCPacketEvents::TimeoutPacketFilter(_)) => {
            todo!()
        }
        IBCHandlerEvents::OwnableEvent(_) => QueueMsg::Noop,
    }
}

pub fn with_channel<Hc, T>(
    chain_id: ChainIdOf<Hc>,
    port_id: String,
    channel_id: String,
    event_height: HeightOf<Hc>,
    tx_hash: H256,
    raw_event: T,
) -> QueueMsg<BlockMessageTypes>
where
    Hc: ChainExt + EthereumChain,

    AggregateWithChannel<Hc>: From<EventInfo<Hc, T>>,

    Hc::Aggregate: From<AggregateWithChannel<Hc>>,
    Hc::Fetch: From<FetchChannel<Hc>>,

    AnyChainIdentified<AnyAggregate>: From<Identified<Hc, Aggregate<Hc>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Hc, Fetch<Hc>>>,
{
    aggregate(
        [fetch(id(
            chain_id.clone(),
            Fetch::<Hc>::specific(FetchChannel {
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
            Aggregate::<Hc>::specific(AggregateWithChannel::from(EventInfo {
                height: event_height,
                tx_hash,
                raw_event,
            })),
        ),
    )
}

pub fn with_connection<Hc, T>(
    chain_id: ChainIdOf<Hc>,
    connection_id: String,
    event_height: HeightOf<Hc>,
    tx_hash: H256,
    raw_event: T,
) -> QueueMsg<BlockMessageTypes>
where
    Hc: ChainExt + EthereumChain,

    AggregateWithConnection<Hc>: From<EventInfo<Hc, T>>,

    Hc::Aggregate: From<AggregateWithConnection<Hc>>,
    Hc::Fetch: From<FetchConnection<Hc>>,

    AnyChainIdentified<AnyAggregate>: From<Identified<Hc, Aggregate<Hc>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Hc, Fetch<Hc>>>,
{
    aggregate(
        [fetch(id(
            chain_id.clone(),
            Fetch::<Hc>::specific(FetchConnection {
                height: event_height,
                path: ConnectionPath {
                    connection_id: connection_id.parse().unwrap(),
                },
            }),
        ))],
        [],
        id(
            chain_id,
            Aggregate::<Hc>::specific(AggregateWithConnection::from(EventInfo {
                height: event_height,
                tx_hash,
                raw_event,
            })),
        ),
    )
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum EthereumFetch<C: ChainSpec> {
    FetchEvents(FetchEvents<C>),
    FetchGetLogs(FetchGetLogs),
    FetchBeaconBlockRange(FetchBeaconBlockRange),

    FetchChannel(FetchChannel<Ethereum<C>>),
    FetchConnection(FetchConnection<Ethereum<C>>),
}

#[queue_msg]
pub struct FetchEvents<C: ChainSpec> {
    pub from_height: HeightOf<Ethereum<C>>,
    pub to_height: HeightOf<Ethereum<C>>,
}

#[queue_msg]
pub struct FetchGetLogs {
    pub from_slot: u64,
    pub to_slot: u64,
}

#[queue_msg]
/// NOTE: This isn't just fetching one block because sometimes beacon slots are missed. We need to be able to fetch a range of slots to account for this.
/// The range is `[from_slot..to_slot)`, so to fetch a single block `N`, the range would be `N..N+1`.
pub struct FetchBeaconBlockRange {
    pub from_slot: u64,
    pub to_slot: u64,
}

#[queue_msg]
pub struct FetchChannel<Hc: EthereumChainExt> {
    pub height: Hc::Height,
    pub path: ChannelEndPath,
}

#[queue_msg]
pub struct FetchConnection<Hc: EthereumChainExt> {
    pub height: Hc::Height,
    pub path: ConnectionPath,
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum EthereumAggregate<C: ChainSpec> {
    AggregateWithChannel(AggregateWithChannel<Ethereum<C>>),
    AggregateWithConnection(AggregateWithConnection<Ethereum<C>>),
}

impl<C: ChainSpec> DoAggregate for Identified<Ethereum<C>, EthereumAggregate<C>>
where
    AnyChainIdentified<AnyData>: From<Identified<Ethereum<C>, ChainEvent<Ethereum<C>>>>,

    Identified<Ethereum<C>, ChannelData<Ethereum<C>>>: IsAggregateData,
    Identified<Ethereum<C>, ConnectionData<Ethereum<C>>>: IsAggregateData,
{
    fn do_aggregate(
        Identified { chain_id, t }: Self,
        data: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> QueueMsg<BlockMessageTypes> {
        match t {
            EthereumAggregate::AggregateWithChannel(msg) => {
                do_aggregate(id::<Ethereum<C>, _>(chain_id, msg), data)
            }
            EthereumAggregate::AggregateWithConnection(msg) => {
                do_aggregate(id::<Ethereum<C>, _>(chain_id, msg), data)
            }
        }
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum AggregateWithChannel<Hc: ChainExt + EthereumChain> {
    PacketAcknowledgement(EventInfo<Hc, AcknowledgePacketFilter>),
    SendPacket(EventInfo<Hc, SendPacketFilter>),
    RecvPacket(EventInfo<Hc, RecvPacketFilter>),
    ChannelOpenInit(EventInfo<Hc, ChannelOpenInitFilter>),
    ChannelOpenTry(EventInfo<Hc, ChannelOpenTryFilter>),
    ChannelOpenAck(EventInfo<Hc, ChannelOpenAckFilter>),
    ChannelOpenConfirm(EventInfo<Hc, ChannelOpenConfirmFilter>),
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum AggregateWithConnection<Hc: ChainExt + EthereumChain> {
    ConnectionOpenInit(EventInfo<Hc, ConnectionOpenInitFilter>),
    ConnectionOpenTry(EventInfo<Hc, ConnectionOpenTryFilter>),
    ConnectionOpenAck(EventInfo<Hc, ConnectionOpenAckFilter>),
    ConnectionOpenConfirm(EventInfo<Hc, ConnectionOpenConfirmFilter>),
}

#[derive(macros::Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(
    rename_all = "snake_case",
    deny_unknown_fields,
    bound(
        serialize = "T: Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    )
)]
pub struct EventInfo<Hc: ChainExt + EthereumChain, T> {
    height: Hc::Height,
    tx_hash: H256,
    raw_event: T,
}

impl<Hc: ChainExt + EthereumChain, T: PartialEq> PartialEq for EventInfo<Hc, T> {
    fn eq(&self, other: &Self) -> bool {
        self.height == other.height
            && self.tx_hash == other.tx_hash
            && self.raw_event == other.raw_event
    }
}

impl<Hc: ChainExt + EthereumChain, T: Clone> Clone for EventInfo<Hc, T> {
    fn clone(&self) -> Self {
        Self {
            height: self.height,
            tx_hash: self.tx_hash,
            raw_event: self.raw_event.clone(),
        }
    }
}

// NOTE: Currently, we assume that EthereumChains will only connect to Union, and as such hardcode the client_type to be Cometbls. This avoids an extra fetch and aggregation to figure out the client type.
impl<Hc: ChainExt + EthereumChain> UseAggregate<BlockMessageTypes>
    for Identified<Hc, AggregateWithChannel<Hc>>
where
    Identified<Hc, ChannelData<Hc>>: IsAggregateData,

    AnyChainIdentified<AnyData>: From<Identified<Hc, ChainEvent<Hc>>>,
{
    type AggregatedData = HList![Identified<Hc, ChannelData<Hc>>];

    fn aggregate(
        Identified { t: msg, chain_id }: Self,
        hlist_pat![Identified {
            chain_id: channel_data_chain_id,
            t: ChannelData {
                channel,
                __marker: _
            }
        }]: Self::AggregatedData,
    ) -> QueueMsg<BlockMessageTypes> {
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

        data(id::<Hc, _>(chain_id, event))
    }
}

impl<Hc: ChainExt + EthereumChain> UseAggregate<BlockMessageTypes>
    for Identified<Hc, AggregateWithConnection<Hc>>
where
    Identified<Hc, ConnectionData<Hc>>: IsAggregateData,

    AnyChainIdentified<AnyData>: From<Identified<Hc, ChainEvent<Hc>>>,
{
    type AggregatedData = HList![Identified<Hc, ConnectionData<Hc>>];

    fn aggregate(
        Identified { t: msg, chain_id }: Self,
        hlist_pat![Identified {
            chain_id: connection_data_chain_id,
            t: ConnectionData(connection)
        }]: Self::AggregatedData,
    ) -> QueueMsg<BlockMessageTypes> {
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

        data(id::<Hc, _>(chain_id, event))
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum EthereumData<C: ChainSpec> {
    Channel(ChannelData<Ethereum<C>>),
    Connection(ConnectionData<Ethereum<C>>),
}

const _: () = {
    try_from_block_poll_msg! {
        chain = Ethereum<C>,
        generics = (C: ChainSpec),
        msgs = EthereumData(
            Channel(ChannelData<Ethereum<C>>),
            Connection(ConnectionData<Ethereum<C>>),
        ),
    }
};

#[queue_msg]
pub struct ChannelData<#[cover] Hc: EthereumChainExt> {
    pub channel: Channel,
}

#[queue_msg]
// REVIEW: Use something other than string here?
pub struct ConnectionData<Hc: EthereumChainExt>(pub ConnectionEnd<ClientIdOf<Hc>, String, String>);
