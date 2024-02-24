use std::{collections::VecDeque, fmt::Display};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use frunk::{hlist_pat, HList};
use macros::apply;
use queue_msg::{
    aggregation::{do_aggregate, UseAggregate},
    fetch, HandleAggregate, QueueMsg, QueueMsgTypes,
};
use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::client::height::IsHeight;

use crate::{
    any_enum,
    data::{AnyData, LatestHeight},
    fetch::{AnyFetch, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockPollingTypes, ChainExt, DoAggregate, Identified, IsAggregateData,
};

#[apply(any_enum)]
#[any = AnyAggregate]
pub enum Aggregate<C: ChainExt> {
    FetchBlockRange(AggregateFetchBlockRange<C>),
    #[serde(untagged)]
    ChainSpecific(ChainSpecificAggregate<C>),
}

impl<C: ChainExt> Display for Aggregate<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aggregate::FetchBlockRange(fetch_block_range) => {
                write!(f, "FetchBlockRange({}..)", fetch_block_range.from_height)
            }
            Aggregate::ChainSpecific(agg) => write!(f, "ChainSpecific({})", agg.0),
        }
    }
}

impl HandleAggregate<BlockPollingTypes> for AnyChainIdentified<AnyAggregate> {
    fn handle(
        self,
        data: VecDeque<<BlockPollingTypes as QueueMsgTypes>::Data>,
    ) -> queue_msg::QueueMsg<BlockPollingTypes> {
        match self {
            AnyChainIdentified::Cosmos(aggregate) => aggregate.handle(data),
            AnyChainIdentified::Union(aggregate) => aggregate.handle(data),
            AnyChainIdentified::EvmMainnet(aggregate) => aggregate.handle(data),
            AnyChainIdentified::EvmMinimal(aggregate) => aggregate.handle(data),
        }
    }
}

impl<C: ChainExt> Identified<C, Aggregate<C>> {
    pub fn handle(self, data: VecDeque<AnyChainIdentified<AnyData>>) -> QueueMsg<BlockPollingTypes>
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

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainExt")
)]
pub struct ChainSpecificAggregate<C: ChainExt>(pub C::Aggregate);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainExt")
)]
pub struct AggregateFetchBlockRange<C: ChainExt> {
    pub from_height: C::Height,
}

impl<C: ChainExt> UseAggregate<BlockPollingTypes> for Identified<C, AggregateFetchBlockRange<C>>
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
    ) -> QueueMsg<BlockPollingTypes> {
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
