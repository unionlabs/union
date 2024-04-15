use std::collections::VecDeque;

use frunk::{hlist_pat, HList};
use macros::apply;
use queue_msg::{
    aggregation::{do_aggregate, UseAggregate},
    fetch, queue_msg, HandleAggregate, QueueError, QueueMessageTypes, QueueMsg,
};
use unionlabs::ibc::core::client::height::IsHeight;

use crate::{
    any_chain, any_enum,
    data::{AnyData, LatestHeight},
    fetch::{AnyFetch, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockMessageTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
};

#[apply(any_enum)]
#[any = AnyAggregate]
#[specific = ChainSpecificAggregate]
pub enum Aggregate<C: ChainExt> {
    FetchBlockRange(AggregateFetchBlockRange<C>),
    #[serde(untagged)]
    ChainSpecific(ChainSpecificAggregate<C>),
}

impl HandleAggregate<BlockMessageTypes> for AnyChainIdentified<AnyAggregate> {
    fn handle(
        self,
        data: VecDeque<<BlockMessageTypes as QueueMessageTypes>::Data>,
    ) -> Result<QueueMsg<BlockMessageTypes>, QueueError> {
        let aggregate = self;

        any_chain! {
            |aggregate| Ok(aggregate.handle(data))
        }
    }
}

impl<C: ChainExt> Identified<C, Aggregate<C>> {
    pub fn handle(self, data: VecDeque<AnyChainIdentified<AnyData>>) -> QueueMsg<BlockMessageTypes>
    where
        Identified<C, C::Aggregate>: DoAggregate,

        Identified<C, LatestHeight<C>>: IsAggregateData,

        AnyChainIdentified<AnyFetch>: From<Identified<C, Fetch<C>>>,
    {
        let chain_id = self.chain_id;

        match self.t {
            Aggregate::ChainSpecific(ChainSpecificAggregate(aggregate)) => {
                <Identified<_, C::Aggregate> as DoAggregate>::do_aggregate(
                    id(chain_id, aggregate),
                    data,
                )
            }
            Aggregate::FetchBlockRange(aggregate) => do_aggregate(id(chain_id, aggregate), data),
        }
    }
}

#[queue_msg]
pub struct ChainSpecificAggregate<C: ChainExt>(pub C::Aggregate);

#[queue_msg]
pub struct AggregateFetchBlockRange<C: ChainExt> {
    pub from_height: C::Height,
}

impl<C: ChainExt> UseAggregate<BlockMessageTypes> for Identified<C, AggregateFetchBlockRange<C>>
where
    Identified<C, LatestHeight<C>>: IsAggregateData,

    AnyChainIdentified<AnyFetch>: From<Identified<C, Fetch<C>>>,
{
    type AggregatedData = HList![Identified<C, LatestHeight<C>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t: AggregateFetchBlockRange { from_height },
        }: Self,
        hlist_pat![Identified {
            chain_id: latest_height_chain_id,
            t: LatestHeight(to_height),
        }]: Self::AggregatedData,
    ) -> QueueMsg<BlockMessageTypes> {
        assert!(to_height.revision_height() > from_height.revision_number());
        assert_eq!(this_chain_id, latest_height_chain_id);

        fetch(Identified::<C, _>::new(
            this_chain_id,
            FetchBlockRange {
                from_height,
                to_height,
            },
        ))
    }
}
