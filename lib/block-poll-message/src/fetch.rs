use std::fmt::{Debug, Display};

use chain_utils::GetChain;
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use futures::Future;
use macros::apply;
use queue_msg::{aggregate, fetch, seq, wait, HandleFetch, QueueMsg, QueueMsgTypes};
use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::client::height::IsHeight;

use crate::{
    aggregate::{Aggregate, AggregateFetchBlockRange, AnyAggregate},
    any_enum,
    wait::{AnyWait, Wait, WaitForHeight},
    AnyChainIdentified, BlockPollingTypes, ChainExt, Identified,
};

#[apply(any_enum)]
#[any = AnyFetch]
pub enum Fetch<C: ChainExt> {
    FetchBlock(FetchBlock<C>),
    FetchBlockRange(FetchBlockRange<C>),

    #[serde(untagged)]
    ChainSpecific(ChainSpecificFetch<C>),
}

impl<C: ChainExt> Display for Fetch<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fetch::FetchBlock(fb) => write!(f, "FetchBlock({})", fb.height),
            Fetch::FetchBlockRange(fbr) => {
                write!(f, "FetchBlockRange({}..{})", fbr.from_height, fbr.to_height)
            }
            Fetch::ChainSpecific(cs) => write!(f, "{}", cs.0),
        }
    }
}

impl HandleFetch<BlockPollingTypes> for AnyChainIdentified<AnyFetch> {
    async fn handle(
        self,
        store: &<BlockPollingTypes as QueueMsgTypes>::Store,
    ) -> QueueMsg<BlockPollingTypes> {
        match self {
            AnyChainIdentified::Cosmos(fetch) => {
                fetch.t.handle(store.get_chain(&fetch.chain_id)).await
            }
            AnyChainIdentified::Union(fetch) => {
                fetch.t.handle(store.get_chain(&fetch.chain_id)).await
            }
            AnyChainIdentified::EvmMainnet(fetch) => {
                fetch.t.handle(store.get_chain(&fetch.chain_id)).await
            }
            AnyChainIdentified::EvmMinimal(fetch) => {
                fetch.t.handle(store.get_chain(&fetch.chain_id)).await
            }
            AnyChainIdentified::Scroll(fetch) => {
                fetch.t.handle(store.get_chain(&fetch.chain_id)).await
            }
        }
    }
}

impl<C> Fetch<C>
where
    C: ChainExt + DoFetchBlockRange<C>,
    C::Fetch: DoFetch<C>,
    // AnyChainIdentified<AnyData>: From<identified!(Data<C>)>,
    AnyChainIdentified<AnyFetch>: From<Identified<C, Fetch<C>>>,
    AnyChainIdentified<AnyWait>: From<Identified<C, Wait<C>>>,
    AnyChainIdentified<AnyAggregate>: From<Identified<C, Aggregate<C>>>,
{
    pub async fn handle(self, c: C) -> QueueMsg<BlockPollingTypes> {
        match self {
            Fetch::FetchBlock(FetchBlock { height }) => aggregate(
                [wait(Identified::<C, _>::new(
                    c.chain_id(),
                    WaitForHeight {
                        height: height.increment(),
                    },
                ))],
                [],
                Identified::<C, _>::new(
                    c.chain_id(),
                    AggregateFetchBlockRange {
                        from_height: height,
                    },
                ),
            ),
            Fetch::FetchBlockRange(range) => seq([
                C::fetch_block_range(&c, range.clone()),
                fetch(Identified::<C, _>::new(
                    c.chain_id(),
                    FetchBlock {
                        height: range.to_height,
                    },
                )),
            ]),
            Fetch::ChainSpecific(cs) => C::Fetch::do_fetch(&c, cs.0).await,
        }
    }
}

pub trait DoFetch<C: ChainExt>: Sized + Debug + Clone + PartialEq {
    fn do_fetch(c: &C, _: Self) -> impl Future<Output = QueueMsg<BlockPollingTypes>>;
}

pub trait DoFetchBlockRange<C: ChainExt>: ChainExt {
    fn fetch_block_range(c: &C, range: FetchBlockRange<C>) -> QueueMsg<BlockPollingTypes>;
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainExt")
)]
pub struct FetchBlockRange<C: ChainExt> {
    pub from_height: C::Height,
    pub to_height: C::Height,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainExt")
)]
pub struct FetchBlock<C: ChainExt> {
    pub height: C::Height,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary),
    arbitrary(bound = "C: ChainExt")
)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct ChainSpecificFetch<C: ChainExt>(pub C::Fetch);
