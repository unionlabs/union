use std::fmt::Debug;

use chain_utils::GetChain;
use futures::Future;
use macros::apply;
use queue_msg::{
    aggregate, conc, fetch, queue_msg, wait, HandleFetch, Op, QueueError, QueueMessage,
};
use tracing::instrument;
use unionlabs::ibc::core::client::height::IsHeight;

use crate::{
    aggregate::{Aggregate, AggregateFetchBlockRange, AnyAggregate},
    any_chain, any_enum,
    wait::{AnyWait, Wait, WaitForHeight},
    AnyChainIdentified, BlockMessage, ChainExt, Identified,
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

fn assert_send<T: Send>(t: T) -> T {
    t
}

impl HandleFetch<BlockMessage> for AnyChainIdentified<AnyFetch> {
    #[instrument(skip_all, fields(chain_id = %self.chain_id()))]
    async fn handle(
        self,
        store: &<BlockMessage as QueueMessage>::Store,
    ) -> Result<Op<BlockMessage>, QueueError> {
        let fetch = self;

        assert_send(any_chain! {
            |fetch| {
                Ok(store
                    .with_chain(&fetch.chain_id, move |c| fetch.t.handle(c))
                    .map_err(|e| QueueError::Fatal(Box::new(e)))?
                    .await)
            }
        })
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
    pub async fn handle(self, c: C) -> Op<BlockMessage> {
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
    fn do_fetch(c: &C, _: Self) -> impl Future<Output = Op<BlockMessage>>;
}

pub trait DoFetchBlockRange<C: ChainExt>: ChainExt {
    fn fetch_block_range(c: &C, range: FetchBlockRange<C>) -> Op<BlockMessage>;
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
