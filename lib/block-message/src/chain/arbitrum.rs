use std::collections::VecDeque;

use chain_utils::{
    arbitrum::{Arbitrum, ARBITRUM_REVISION_NUMBER},
    ethereum::{AnyEthereum, EthereumConsensusChain},
};
use enumorph::Enumorph;
use queue_msg::{aggregation::do_aggregate, fetch, queue_msg, Op};
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

impl ChainExt for Arbitrum {
    type Data = ArbitrumData;
    type Fetch = ArbitrumFetch;
    type Aggregate = ArbitrumAggregate;
}

impl DoFetchBlockRange<Arbitrum> for Arbitrum
where
    AnyChainIdentified<AnyFetch>: From<Identified<Arbitrum, Fetch<Arbitrum>>>,
{
    fn fetch_block_range(c: &Arbitrum, range: FetchBlockRange<Arbitrum>) -> Op<BlockMessage> {
        fetch(id(
            c.chain_id(),
            Fetch::<Arbitrum>::specific(FetchEvents {
                from_height: range.from_height,
                to_height: range.to_height,
            }),
        ))
    }
}

impl DoFetch<Arbitrum> for ArbitrumFetch
where
    AnyChainIdentified<AnyData>: From<Identified<Arbitrum, Data<Arbitrum>>>,
    AnyChainIdentified<AnyAggregate>: From<Identified<Arbitrum, Aggregate<Arbitrum>>>,
    AnyChainIdentified<AnyFetch>: From<Identified<Arbitrum, Fetch<Arbitrum>>>,
{
    async fn do_fetch(c: &Arbitrum, msg: Self) -> Op<BlockMessage> {
        match msg {
            Self::FetchEvents(FetchEvents {
                from_height,
                to_height,
            }) => fetch(id(
                c.chain_id(),
                Fetch::<Arbitrum>::specific(FetchBeaconBlockRange {
                    from_slot: from_height.revision_height,
                    to_slot: to_height.revision_height,
                }),
            )),
            Self::FetchGetLogs(get_logs) => {
                fetch_get_logs(c, get_logs, ARBITRUM_REVISION_NUMBER).await
            }
            Self::FetchBeaconBlockRange(beacon_block_range) => match &c.l1 {
                AnyEthereum::Mainnet(eth) => {
                    fetch_beacon_block_range(c, beacon_block_range, &eth.beacon_api_client).await
                }
                AnyEthereum::Minimal(eth) => {
                    fetch_beacon_block_range(c, beacon_block_range, &eth.beacon_api_client).await
                }
            },
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

#[queue_msg]
#[derive(Enumorph)]
pub enum ArbitrumFetch {
    FetchEvents(FetchEvents<Arbitrum>),
    FetchGetLogs(FetchGetLogs),
    FetchBeaconBlockRange(FetchBeaconBlockRange),

    FetchChannel(FetchChannel<Arbitrum>),
}

#[queue_msg]
pub struct FetchBatchIndex {
    beacon_slot: u64,
    batch_index: u64,
}

#[queue_msg]
#[derive(Enumorph)]
pub enum ArbitrumAggregate {
    AggregateWithChannel(AggregateWithChannel<Arbitrum>),
}

impl DoAggregate for Identified<Arbitrum, ArbitrumAggregate>
where
    AnyChainIdentified<AnyData>: From<Identified<Arbitrum, ChainEvent<Arbitrum>>>,

    Identified<Arbitrum, ChannelData<Arbitrum>>: IsAggregateData,
    Identified<Arbitrum, ConnectionData<Arbitrum>>: IsAggregateData,
{
    fn do_aggregate(
        Identified { chain_id, t }: Self,
        data: VecDeque<AnyChainIdentified<AnyData>>,
    ) -> Op<BlockMessage> {
        match t {
            ArbitrumAggregate::AggregateWithChannel(msg) => do_aggregate(id(chain_id, msg), data),
        }
    }
}

#[queue_msg]
#[derive(Enumorph)]
pub enum ArbitrumData {
    Channel(ChannelData<Arbitrum>),
    Connection(ConnectionData<Arbitrum>),
}

const _: () = {
    try_from_block_poll_msg! {
        chain = Arbitrum,
        generics = (),
        msgs = ArbitrumData(
            Channel(ChannelData<Arbitrum>),
            Connection(ConnectionData<Arbitrum>),
        ),
    }
};
