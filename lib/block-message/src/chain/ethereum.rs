use std::{collections::VecDeque, marker::PhantomData};

use beacon_api::client::BeaconApiClient;
use chain_utils::ethereum::{
    Ethereum, EthereumChain, EthereumConsensusChain, IBCHandlerEvents, IbcHandlerExt,
    ETHEREUM_REVISION_NUMBER,
};
use contracts::{
    ibc_channel_handshake::IBCChannelHandshakeEvents,
    ibc_client::{ClientCreatedFilter, ClientUpdatedFilter, IBCClientEvents},
    ibc_connection::IBCConnectionEvents,
    ibc_packet::{
        AcknowledgePacketFilter, IBCPacketEvents, RecvPacketFilter, SendPacketFilter,
        WriteAcknowledgementFilter,
    },
};
use enumorph::Enumorph;
use ethers::{contract::EthLogDecode, providers::Middleware, types::Filter};
use frunk::{hlist_pat, HList};
use futures::StreamExt;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    conc, data, fetch, noop, queue_msg, QueueMsg,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use unionlabs::{
    ethereum::config::ChainSpec,
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
        CreateClient, IbcEvent, RecvPacket, SendPacket, UpdateClient, WriteAcknowledgement,
    },
    hash::H256,
    ibc::{
        core::{
            channel::channel::Channel,
            client::height::{Height, IsHeight},
            connection::connection_end::ConnectionEnd,
        },
        lightclients::cometbls,
    },
    ics24::ChannelEndPath,
    id::ClientIdValidator,
    traits::{Chain, ChainIdOf, ClientIdOf, HeightOf},
    validated::ValidateT,
};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    data::{AnyData, ChainEvent, Data},
    fetch::{AnyFetch, DoFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockMessage, ChainExt, DoAggregate, Identified, IsAggregateData,
};

pub trait EthereumChainExt = ChainExt + chain_utils::ethereum::EthereumChainExt;

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
    ) -> QueueMsg<BlockMessage> {
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
    async fn do_fetch(c: &Ethereum<C>, msg: Self) -> QueueMsg<BlockMessage> {
        match msg {
            Self::FetchEvents(FetchEvents {
                from_height,
                to_height,
            }) => fetch(id(
                c.chain_id(),
                Fetch::<Ethereum<C>>::specific(FetchBeaconBlockRange {
                    from_slot: from_height.revision_height,
                    to_slot: to_height.revision_height,
                }),
            )),
            Self::FetchGetLogs(get_logs) => {
                fetch_get_logs(c, get_logs, ETHEREUM_REVISION_NUMBER).await
            }
            Self::FetchBeaconBlockRange(beacon_block_range) => {
                fetch_beacon_block_range(c, beacon_block_range, &c.beacon_api_client).await
            }
            Self::FetchChannel(FetchChannel { height, path }) => {
                fetch_channel(
                    c,
                    path,
                    c.execution_height_of_beacon_slot(height.revision_height())
                        .await,
                )
                .await
            }
        }
    }
}

pub(crate) async fn fetch_get_logs<Hc>(
    c: &Hc,
    FetchGetLogs { from_slot, to_slot }: FetchGetLogs,
    revision_number: u64,
) -> QueueMsg<BlockMessage>
where
    Hc: EthereumConsensusChain
        + EthereumChainExt<
            Height = Height,
            Aggregate: From<AggregateWithChannel<Hc>>,
            Fetch: From<FetchChannel<Hc>>,
        >,

    AnyChainIdentified<AnyAggregate>: From<Identified<Hc, Aggregate<Hc>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Hc, Fetch<Hc>>>,
    AnyChainIdentified<AnyData>: From<Identified<Hc, Data<Hc>>>,
{
    debug!(%from_slot, %to_slot, "fetching logs in beacon block range");

    let event_height = Height {
        revision_number,
        revision_height: to_slot,
    };

    let from_block = c.execution_height_of_beacon_slot(from_slot).await;
    let to_block = c.execution_height_of_beacon_slot(to_slot).await;

    if from_block == to_block {
        debug!(%from_block, %to_block, %from_slot, %to_slot, "beacon block range is empty");
        noop()
    } else {
        debug!(%from_block, %to_block, "fetching block range");
        // REVIEW: Surely transactions and events can be fetched in parallel?
        conc(
            futures::stream::iter(
                c.provider()
                    .get_logs(
                        &Filter::new()
                            .address(ethers::types::H160::from(c.ibc_handler_address()))
                            .from_block(from_block)
                            // NOTE: This -1 is very important, else events will be double fetched
                            .to_block(to_block - 1),
                    )
                    .await
                    .unwrap(),
            )
            .filter_map(|log| async {
                let tx_hash = log
                    .transaction_hash
                    .expect("log should have transaction_hash")
                    .into();

                debug!(?log, "raw log");

                match IBCHandlerEvents::decode_log(&log.into()) {
                    Ok(event) => {
                        debug!(?event, "found IBCHandler event");
                        Some(
                            mk_aggregate_event(c, event, event_height, tx_hash, |event_height| {
                                c.execution_height_of_beacon_slot(event_height.revision_height())
                            })
                            .await,
                        )
                    }
                    Err(e) => {
                        warn!("could not decode evm event {}", e);
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .await,
        )
    }
}

pub(crate) async fn fetch_beacon_block_range<C, Hc>(
    c: &Hc,
    FetchBeaconBlockRange { from_slot, to_slot }: FetchBeaconBlockRange,
    beacon_api_client: &BeaconApiClient<C>,
) -> QueueMsg<BlockMessage>
where
    C: ChainSpec,
    Hc: ChainExt<Fetch: From<FetchGetLogs> + From<FetchBeaconBlockRange>> + EthereumChain,

    AnyChainIdentified<AnyFetch>: From<Identified<Hc, Fetch<Hc>>>,
{
    debug!(%from_slot, %to_slot, "fetching beacon block range");

    assert!(from_slot < to_slot);

    if to_slot - from_slot == 1 {
        fetch(id(
            c.chain_id(),
            Fetch::<Hc>::specific(FetchGetLogs { from_slot, to_slot }),
        ))
    } else {
        // attempt to shrink from..to
        // note that this is *exclusive* on `to`
        for slot in (from_slot + 1)..to_slot {
            info!(%slot, "querying slot");

            match beacon_api_client
                .block(beacon_api::client::BlockId::Slot(slot))
                .await
            {
                Err(beacon_api::errors::Error::NotFound(beacon_api::errors::NotFoundError {
                    message,
                    error,
                    status_code,
                })) => {
                    info!(%slot, %message, %error, %status_code, "beacon block not found for slot");
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

pub(crate) async fn fetch_channel<Hc>(
    c: &Hc,
    path: ChannelEndPath,
    execution_height: u64,
) -> QueueMsg<BlockMessage>
where
    Hc: EthereumChainExt<Data: From<ChannelData<Hc>>>,

    AnyChainIdentified<AnyData>: From<Identified<Hc, Data<Hc>>>,
{
    debug!(%execution_height, %path, "fetching channel");

    data(id(
        c.chain_id(),
        Data::<Hc>::specific(ChannelData {
            channel: c
                .ibc_handler()
                .get_channel(path.port_id.to_string(), path.channel_id.to_string())
                .block(execution_height)
                .await
                .unwrap()
                .try_into()
                .unwrap(),
            __marker: PhantomData,
        }),
    ))
}

pub async fn mk_aggregate_event<Hc, F, Fut>(
    c: &Hc,
    event: IBCHandlerEvents,
    event_height: Hc::Height,
    tx_hash: H256,
    // normalize the height from the "public facing" height to the execution height of this chain.
    normalize_height: F,
) -> QueueMsg<BlockMessage>
where
    Hc: EthereumChainExt<Aggregate: From<AggregateWithChannel<Hc>>, Fetch: From<FetchChannel<Hc>>>,

    AnyChainIdentified<AnyAggregate>: From<Identified<Hc, Aggregate<Hc>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Hc, Fetch<Hc>>>,
    AnyChainIdentified<AnyData>: From<Identified<Hc, Data<Hc>>>,

    F: FnOnce(Hc::Height) -> Fut,
    Fut: futures::Future<Output = u64>,
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
        )) => data(id(
            c.chain_id(),
            ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height: event_height,
                event: IbcEvent::ChannelOpenAck(ChannelOpenAck {
                    port_id: raw_event.port_id.parse().unwrap(),
                    channel_id: raw_event.channel_id.parse().unwrap(),
                    counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                    counterparty_channel_id: raw_event.counterparty_channel_id.parse().unwrap(),
                    connection_id: raw_event.connection_id.parse().unwrap(),
                }),
            },
        )),
        IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenConfirmFilter(
            raw_event,
        )) => data(id(
            c.chain_id(),
            ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height: event_height,
                event: IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
                    port_id: raw_event.port_id.parse().unwrap(),
                    channel_id: raw_event.channel_id.parse().unwrap(),
                    counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                    counterparty_channel_id: raw_event.counterparty_channel_id.parse().unwrap(),
                    connection_id: raw_event.connection_id.parse().unwrap(),
                }),
            },
        )),
        IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenInitFilter(
            raw_event,
        )) => data(id(
            c.chain_id(),
            ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height: event_height,
                event: IbcEvent::ChannelOpenInit(ChannelOpenInit {
                    port_id: raw_event.port_id.parse().unwrap(),
                    channel_id: raw_event.channel_id.parse().unwrap(),
                    counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    version: raw_event.version,
                }),
            },
        )),
        IBCHandlerEvents::ChannelEvent(IBCChannelHandshakeEvents::ChannelOpenTryFilter(
            raw_event,
        )) => data(id(
            c.chain_id(),
            ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height: event_height,
                event: IbcEvent::ChannelOpenTry(ChannelOpenTry {
                    port_id: raw_event.port_id.parse().unwrap(),
                    channel_id: raw_event.channel_id.parse().unwrap(),
                    counterparty_port_id: raw_event.counterparty_port_id.parse().unwrap(),
                    counterparty_channel_id: raw_event.counterparty_channel_id.parse().unwrap(),
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    version: raw_event.version,
                }),
            },
        )),
        IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenAckFilter(
            raw_event,
        )) => data(id(
            c.chain_id(),
            ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height: event_height,
                event: IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    client_id: raw_event.client_id.parse().unwrap(),
                    counterparty_client_id: raw_event.counterparty_client_id.parse().unwrap(),
                    counterparty_connection_id: raw_event
                        .counterparty_connection_id
                        .parse()
                        .unwrap(),
                }),
            },
        )),
        IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenConfirmFilter(
            raw_event,
        )) => data(id(
            c.chain_id(),
            ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height: event_height,
                event: IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    client_id: raw_event.client_id.parse().unwrap(),
                    counterparty_client_id: raw_event.counterparty_client_id.parse().unwrap(),
                    counterparty_connection_id: raw_event
                        .counterparty_connection_id
                        .parse()
                        .unwrap(),
                }),
            },
        )),
        IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenInitFilter(
            raw_event,
        )) => data(id(
            c.chain_id(),
            ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height: event_height,
                event: IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    client_id: raw_event.client_id.parse().unwrap(),
                    counterparty_client_id: raw_event.counterparty_client_id.parse().unwrap(),
                }),
            },
        )),
        IBCHandlerEvents::ConnectionEvent(IBCConnectionEvents::ConnectionOpenTryFilter(
            raw_event,
        )) => data(id(
            c.chain_id(),
            ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height: event_height,
                event: IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                    connection_id: raw_event.connection_id.parse().unwrap(),
                    client_id: raw_event.client_id.parse().unwrap(),
                    counterparty_client_id: raw_event.counterparty_client_id.parse().unwrap(),
                    counterparty_connection_id: raw_event
                        .counterparty_connection_id
                        .parse()
                        .unwrap(),
                }),
            },
        )),
        IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientCreatedFilter(
            ClientCreatedFilter { client_id },
        )) => {
            let client_type = c
                .ibc_handler()
                .client_types(client_id.clone())
                .await
                .unwrap();

            let client_state = c
                .ibc_handler()
                .get_client_state::<Hc, cometbls::client_state::ClientState>(
                    client_id
                        .clone()
                        .validate::<ClientIdValidator>()
                        .unwrap()
                        .try_into()
                        .unwrap(),
                    normalize_height(event_height).await,
                )
                .await;

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
        IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientRegisteredFilter(_)) => noop(),
        IBCHandlerEvents::ClientEvent(IBCClientEvents::ClientUpdatedFilter(
            ClientUpdatedFilter {
                client_id,
                height: consensus_height,
            },
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
            with_channel(
                c.chain_id(),
                raw_event.packet.destination_port.clone(),
                raw_event.packet.destination_channel.clone(),
                event_height,
                tx_hash,
                raw_event,
            )
        }
        IBCHandlerEvents::PacketEvent(IBCPacketEvents::TimeoutPacketFilter(_)) => noop(),
        IBCHandlerEvents::OwnableEvent(_) => noop(),
    }
}

pub fn with_channel<Hc, T>(
    chain_id: ChainIdOf<Hc>,
    port_id: String,
    channel_id: String,
    event_height: HeightOf<Hc>,
    tx_hash: H256,
    raw_event: T,
) -> QueueMsg<BlockMessage>
where
    Hc: ChainExt<Aggregate: From<AggregateWithChannel<Hc>>, Fetch: From<FetchChannel<Hc>>>
        + EthereumChain,

    AggregateWithChannel<Hc>: From<EventInfo<Hc, T>>,

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

#[queue_msg]
#[derive(Enumorph)]
pub enum EthereumFetch<C: ChainSpec> {
    FetchEvents(FetchEvents<Ethereum<C>>),
    FetchGetLogs(FetchGetLogs),
    FetchBeaconBlockRange(FetchBeaconBlockRange),

    FetchChannel(FetchChannel<Ethereum<C>>),
}

#[queue_msg]
pub struct FetchEvents<Hc: ChainExt> {
    pub from_height: HeightOf<Hc>,
    pub to_height: HeightOf<Hc>,
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
// TODO: Move to Data?
pub struct FetchChannel<Hc: ChainExt> {
    pub height: Hc::Height,
    pub path: ChannelEndPath,
}

#[queue_msg]
#[derive(Enumorph)]
pub enum EthereumAggregate<C: ChainSpec> {
    AggregateWithChannel(AggregateWithChannel<Ethereum<C>>),
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
    ) -> QueueMsg<BlockMessage> {
        match t {
            EthereumAggregate::AggregateWithChannel(msg) => {
                do_aggregate(id::<Ethereum<C>, _>(chain_id, msg), data)
            }
        }
    }
}

#[queue_msg]
#[derive(Enumorph)]
pub enum AggregateWithChannel<Hc: ChainExt> {
    PacketAcknowledgement(EventInfo<Hc, AcknowledgePacketFilter>),
    WriteAcknowledgement(EventInfo<Hc, WriteAcknowledgementFilter>),
    SendPacket(EventInfo<Hc, SendPacketFilter>),
    RecvPacket(EventInfo<Hc, RecvPacketFilter>),
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
// REVIEW: Use something like derivative/ derive_where/ educe
pub struct EventInfo<Hc: ChainExt, T> {
    height: Hc::Height,
    tx_hash: H256,
    raw_event: T,
}

impl<Hc: ChainExt, T: PartialEq> PartialEq for EventInfo<Hc, T> {
    fn eq(&self, other: &Self) -> bool {
        self.height == other.height
            && self.tx_hash == other.tx_hash
            && self.raw_event == other.raw_event
    }
}

impl<Hc: ChainExt, T: Clone> Clone for EventInfo<Hc, T> {
    fn clone(&self) -> Self {
        Self {
            height: self.height,
            tx_hash: self.tx_hash,
            raw_event: self.raw_event.clone(),
        }
    }
}

// NOTE: Currently, we assume that EthereumChains will only connect to Union, and as such hardcode the client_type to be Cometbls. This avoids an extra fetch and aggregation to figure out the client type.
impl<Hc: ChainExt + EthereumChain> UseAggregate<BlockMessage>
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
    ) -> QueueMsg<BlockMessage> {
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
            AggregateWithChannel::WriteAcknowledgement(EventInfo {
                height,
                tx_hash,
                raw_event,
            }) => ChainEvent {
                client_type: unionlabs::ClientType::Cometbls,
                tx_hash,
                height,
                event: IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
                    packet_data_hex: raw_event.packet.data.to_vec(),
                    packet_timeout_height: raw_event.packet.timeout_height.into(),
                    packet_timeout_timestamp: raw_event.packet.timeout_timestamp,
                    packet_sequence: raw_event.packet.sequence.try_into().unwrap(),
                    packet_src_port: raw_event.packet.source_port.parse().unwrap(),
                    packet_src_channel: raw_event.packet.source_channel.parse().unwrap(),
                    packet_dst_port: raw_event.packet.destination_port.parse().unwrap(),
                    packet_dst_channel: raw_event.packet.destination_channel.parse().unwrap(),
                    packet_ack_hex: raw_event.acknowledgement.to_vec(),
                    connection_id: channel.connection_hops[0].clone(),
                }),
            },
        };

        data(id::<Hc, _>(chain_id, event))
    }
}

#[queue_msg]
#[derive(Enumorph)]
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
pub struct ChannelData<#[cover] Hc: ChainExt> {
    pub channel: Channel,
}

#[queue_msg]
// REVIEW: Use something other than string here?
pub struct ConnectionData<Hc: ChainExt>(pub ConnectionEnd<ClientIdOf<Hc>, String, String>);
