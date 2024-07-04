use chain_utils::near::Near;
use queue_msg::Op;
use unionlabs::never::Never;

use crate::{
    fetch::{AnyFetch, DoFetchBlockRange, Fetch, FetchBlockRange},
    AnyChainIdentified, BlockMessage, ChainExt, Identified,
};

impl ChainExt for Near {
    type Data = Never;

    type Fetch = Never;

    type Aggregate = Never;
}

impl DoFetchBlockRange<Near> for Near
where
    AnyChainIdentified<AnyFetch>: From<Identified<Near, Fetch<Near>>>,
{
    fn fetch_block_range(c: &Near, range: FetchBlockRange<Near>) -> Op<BlockMessage> {
        // fetch(id(
        //     c.chain_id(),
        //     Fetch::<Near>::specific(FetchEvents {
        //         from_height: range.from_height,
        //         to_height: range.to_height,
        //     }),
        // ))
        todo!()
    }
}
