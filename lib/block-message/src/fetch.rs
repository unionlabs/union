use std::fmt::Debug;

use chain_utils::GetChain;
use futures::Future;
use macros::apply;
use queue_msg::{
    aggregate, conc, fetch, queue_msg, wait, HandleFetch, QueueError, QueueMessageTypes, QueueMsg,
};
use unionlabs::ibc::core::client::height::IsHeight;

use crate::{
    aggregate::{Aggregate, AggregateFetchBlockRange, AnyAggregate},
    any_chain, any_enum,
    wait::{AnyWait, Wait, WaitForHeight},
    AnyChainIdentified, BlockMessageTypes, ChainExt, Identified,
};

#[apply(any_enum)]
#[any = AnyFetch]
#[specific = ChainSpecificFetch]
pub enum Fetch<C: ChainExt> {
    FetchBlock(FetchBlock<C>),
    FetchBlockRange(FetchBlockRange<C>),

    #[serde(untagged)]
    ChainSpecific(ChainSpecificFetch<C>),
}

impl HandleFetch<BlockMessageTypes> for AnyChainIdentified<AnyFetch> {
    #[tracing::instrument(skip_all, fields(chain_id = %self.chain_id()))]
    async fn handle(
        self,
        store: &<BlockMessageTypes as QueueMessageTypes>::Store,
    ) -> Result<QueueMsg<BlockMessageTypes>, QueueError> {
        let fetch = self;

        any_chain! {
            |fetch| {
                Ok(store
                    .with_chain(&fetch.chain_id, move |c| fetch.t.handle(c))
                    .map_err(|e| QueueError::Fatal(Box::new(e)))?
                    .await)
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
    pub async fn handle(self, c: C) -> QueueMsg<BlockMessageTypes> {
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
            Fetch::FetchBlockRange(range) => conc([
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
    fn do_fetch(c: &C, _: Self) -> impl Future<Output = QueueMsg<BlockMessageTypes>>;
}

pub trait DoFetchBlockRange<C: ChainExt>: ChainExt {
    fn fetch_block_range(c: &C, range: FetchBlockRange<C>) -> QueueMsg<BlockMessageTypes>;
}

#[queue_msg]
pub struct FetchBlockRange<C: ChainExt> {
    pub from_height: C::Height,
    pub to_height: C::Height,
}

#[queue_msg]
pub struct FetchBlock<C: ChainExt> {
    pub height: C::Height,
}

#[queue_msg]
pub struct ChainSpecificFetch<C: ChainExt>(pub C::Fetch);
