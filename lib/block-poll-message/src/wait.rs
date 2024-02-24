use std::fmt::Display;

use chain_utils::{Chains, GetChain};
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use macros::apply;
use queue_msg::{data, defer, now, seq, wait, HandleWait, QueueMsg};
use serde::{Deserialize, Serialize};
use unionlabs::{ibc::core::client::height::IsHeight, traits::HeightOf};

use crate::{
    any_enum,
    data::{AnyData, Data, LatestHeight},
    AnyChainIdentified, BlockPollingTypes, ChainExt, Identified,
};

#[apply(any_enum)]
#[any = AnyWait]
pub enum Wait<C: ChainExt> {
    Height(WaitForHeight<C>),
}

impl<C: ChainExt> Wait<C>
where
    AnyChainIdentified<AnyWait>: From<Identified<C, Wait<C>>>,
    AnyChainIdentified<AnyData>: From<Identified<C, Data<C>>>,
{
    async fn handle(self, c: &C) -> QueueMsg<BlockPollingTypes> {
        match self {
            Wait::Height(WaitForHeight { height }) => {
                let chain_height = c.query_latest_height().await.unwrap();

                assert_eq!(
                    chain_height.revision_number(),
                    height.revision_number(),
                    "chain_height: {chain_height}, height: {height}",
                );

                tracing::info!("latest height is {chain_height}, waiting for {height}");

                if chain_height.revision_height() >= height.revision_height() {
                    data(Identified::<C, _>::new(
                        c.chain_id(),
                        LatestHeight(chain_height),
                    ))
                } else {
                    seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new method on chain
                        defer(now() + 1),
                        wait(Identified::<C, _>::new(
                            c.chain_id(),
                            WaitForHeight { height },
                        )),
                    ])
                }
            }
        }
    }
}

impl<C: ChainExt> Display for Wait<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wait::Height(h) => write!(f, "Height({})", h.height),
        }
    }
}

impl HandleWait<BlockPollingTypes> for AnyChainIdentified<AnyWait> {
    async fn handle(self, c: &Chains) -> QueueMsg<BlockPollingTypes> {
        match self {
            AnyChainIdentified::Cosmos(w) => w.t.handle(&c.get_chain(&w.chain_id)).await,
            AnyChainIdentified::Union(w) => w.t.handle(&c.get_chain(&w.chain_id)).await,
            AnyChainIdentified::EvmMainnet(w) => w.t.handle(&c.get_chain(&w.chain_id)).await,
            AnyChainIdentified::EvmMinimal(w) => w.t.handle(&c.get_chain(&w.chain_id)).await,
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
pub struct WaitForHeight<C: ChainExt> {
    pub height: HeightOf<C>,
}
