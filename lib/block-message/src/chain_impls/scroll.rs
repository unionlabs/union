use std::{collections::VecDeque, marker::PhantomData};

use chain_utils::{
    ethereum::{IBCHandlerEvents, IbcHandlerExt},
    scroll::{Scroll, SCROLL_REVISION_NUMBER},
};
use contracts::ibc_handler::{GetChannelCall, GetConnectionCall, IBCHandler};
use ethers::{contract::EthLogDecode, providers::Middleware, types::Filter};
use futures::StreamExt;
use queue_msg::{aggregation::do_aggregate, conc, data, fetch, queue_msg, QueueMsg};
use unionlabs::{ethereum::config::Mainnet, ibc::core::client::height::Height, traits::Chain};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain_impls::ethereum::{
        fetch_beacon_block_range, mk_aggregate_event, AggregateWithChannel,
        AggregateWithConnection, ChannelData, ConnectionData, FetchBeaconBlockRange, FetchChannel,
        FetchConnection, FetchEvents, FetchGetLogs,
    },
    data::{AnyData, ChainEvent, Data},
    fetch::{AnyFetch, DoFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockMessageTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
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
    ) -> QueueMsg<BlockMessageTypes> {
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
    async fn do_fetch(scroll: &Scroll, msg: Self) -> QueueMsg<BlockMessageTypes> {
        match msg {
            ScrollFetch::FetchEvents(FetchEvents {
                from_height,
                to_height,
            }) => fetch(id(
                scroll.chain_id(),
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

                let from_batch_index = scroll.batch_index_of_beacon_slot(from_slot).await;
                let to_batch_index = scroll.batch_index_of_beacon_slot(to_slot).await;

                tracing::debug!("slot range {from_slot}..{to_slot} is batch index range {from_batch_index}..{to_batch_index}");

                assert!(from_batch_index <= to_batch_index);

                if from_batch_index == to_batch_index {
                    QueueMsg::Noop
                } else {
                    assert!(from_batch_index + 1 == to_batch_index);

                    let from_scroll_height =
                        scroll.scroll_height_of_batch_index(from_batch_index).await;
                    let to_scroll_height =
                        scroll.scroll_height_of_batch_index(to_batch_index).await;

                    tracing::debug!("batch index {from_slot}..{to_slot} is batch index range {from_batch_index}..{to_batch_index}");

                    conc(
                        futures::stream::iter(
                            scroll
                                .provider
                                .get_logs(
                                    &Filter::new()
                                        .address(ethers::types::H160::from(
                                            scroll.ibc_handler_address,
                                        ))
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

                            match IBCHandlerEvents::decode_log(&log.into()) {
                                Ok(event) => {
                                    mk_aggregate_event(scroll, event, event_height, tx_hash).await
                                }
                                Err(err) => {
                                    tracing::warn!(?err, "failed to decode ibc handler event");
                                    QueueMsg::Noop
                                }
                            }
                        })
                        .collect::<Vec<_>>()
                        .await,
                    )
                }
            }
            ScrollFetch::FetchBeaconBlockRange(beacon_block_range) => {
                fetch_beacon_block_range(scroll, beacon_block_range, &scroll.l1.beacon_api_client)
                    .await
            }
            ScrollFetch::FetchChannel(FetchChannel { height, path }) => {
                let ibc_handler = IBCHandler::new(scroll.ibc_handler_address, scroll.provider.clone());
                let batch_index = scroll
                    .batch_index_of_beacon_slot(height.revision_height)
                    .await;
                let scroll_height = scroll.scroll_height_of_batch_index(batch_index).await;
                data(id(
                    scroll.chain_id(),
                    Data::<Scroll>::specific(ChannelData {
                        channel: ibc_handler
                            .eth_call(
                                GetChannelCall {
                                    port_id: path.port_id.to_string(),
                                    channel_id: path.channel_id.to_string(),
                                },
                                scroll_height,
                            )
                            .await
                            .unwrap()
                            .unwrap()
                            .try_into()
                            .unwrap(),
                        __marker: PhantomData,
                    }),
                ))
            }
            ScrollFetch::FetchConnection(FetchConnection { height, path }) => {
                let ibc_handler = IBCHandler::new(scroll.ibc_handler_address, scroll.provider.clone());
                let batch_index = scroll
                    .batch_index_of_beacon_slot(height.revision_height)
                    .await;
                let scroll_height = scroll.scroll_height_of_batch_index(batch_index).await;
                data(id(
                    scroll.chain_id(),
                    Data::<Scroll>::specific(ConnectionData(
                        ibc_handler
                            .eth_call(
                                GetConnectionCall {
                                    connection_id: path.connection_id.to_string(),
                                },
                                scroll_height,
                            )
                            .await
                            .unwrap()
                            .unwrap()
                            .try_into()
                            .unwrap(),
                    )),
                ))
            }
        }
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ScrollFetch {
    FetchEvents(FetchEvents<Mainnet>),
    FetchGetLogs(FetchGetLogs),
    FetchBeaconBlockRange(FetchBeaconBlockRange),

    FetchChannel(FetchChannel<Scroll>),
    FetchConnection(FetchConnection<Scroll>),
}

#[queue_msg]
pub struct FetchBatchIndex {
    beacon_slot: u64,
    batch_index: u64,
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ScrollAggregate {
    AggregateWithChannel(AggregateWithChannel<Scroll>),
    AggregateWithConnection(AggregateWithConnection<Scroll>),
}

impl DoAggregate for Identified<Scroll, ScrollAggregate>
where
    AnyChainIdentified<AnyData>: From<Identified<Scroll, ChainEvent<Scroll>>>,

    Identified<Scroll, ChannelData<Scroll>>: IsAggregateData,
    Identified<Scroll, ConnectionData<Scroll>>: IsAggregateData,
{
    fn do_aggregate(
        Identified { chain_id, t }: Self,
        data: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> QueueMsg<BlockMessageTypes> {
        match t {
            ScrollAggregate::AggregateWithChannel(msg) => do_aggregate(id(chain_id, msg), data),
            ScrollAggregate::AggregateWithConnection(msg) => do_aggregate(id(chain_id, msg), data),
        }
    }
}

#[queue_msg]
#[derive(enumorph::Enumorph)]
pub enum ScrollData {
    Channel(ChannelData<Scroll>),
    Connection(ConnectionData<Scroll>),
}

const _: () = {
    try_from_block_poll_msg! {
        chain = Scroll,
        generics = (),
        msgs = ScrollData(
            Channel(ChannelData<Scroll>),
            Connection(ConnectionData<Scroll>),
        ),
    }
};
