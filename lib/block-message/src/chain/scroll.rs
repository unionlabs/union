use std::collections::VecDeque;

use chain_utils::{
    ethereum::EthereumConsensusChain,
    scroll::{Scroll, SCROLL_REVISION_NUMBER},
};
use enumorph::Enumorph;
use queue_msg::{aggregation::do_aggregate, fetch, queue_msg, QueueMsg};
use unionlabs::{ibc::core::client::height::IsHeight, traits::Chain};

use crate::{
    aggregate::{Aggregate, AnyAggregate},
    chain::ethereum::{
        fetch_beacon_block_range, fetch_channel, fetch_get_logs, AggregateWithChannel, ChannelData,
        ConnectionData, FetchBeaconBlockRange, FetchChannel, FetchEvents, FetchGetLogs,
    },
    data::{AnyData, ChainEvent, Data},
    fetch::{AnyFetch, DoFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockMessage, ChainExt, DoAggregate, Identified, IsAggregateData,
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
    fn fetch_block_range(c: &Scroll, range: FetchBlockRange<Scroll>) -> QueueMsg<BlockMessage> {
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
    async fn do_fetch(c: &Scroll, msg: Self) -> QueueMsg<BlockMessage> {
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
            ScrollFetch::FetchGetLogs(get_logs) => {
                fetch_get_logs(c, get_logs, SCROLL_REVISION_NUMBER).await
            }
            ScrollFetch::FetchBeaconBlockRange(beacon_block_range) => {
                fetch_beacon_block_range(c, beacon_block_range, &c.l1.beacon_api_client).await
            }
            ScrollFetch::FetchChannel(FetchChannel { height, path }) => {
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

#[queue_msg]
#[derive(Enumorph)]
pub enum ScrollFetch {
    FetchEvents(FetchEvents<Scroll>),
    FetchGetLogs(FetchGetLogs),
    FetchBeaconBlockRange(FetchBeaconBlockRange),

    FetchChannel(FetchChannel<Scroll>),
}

#[queue_msg]
pub struct FetchBatchIndex {
    beacon_slot: u64,
    batch_index: u64,
}

#[queue_msg]
#[derive(Enumorph)]
pub enum ScrollAggregate {
    AggregateWithChannel(AggregateWithChannel<Scroll>),
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
    ) -> QueueMsg<BlockMessage> {
        match t {
            ScrollAggregate::AggregateWithChannel(msg) => do_aggregate(id(chain_id, msg), data),
        }
    }
}

#[queue_msg]
#[derive(Enumorph)]
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
