use chain_utils::scroll::Scroll;
use unionlabs::never::Never;

use crate::ChainExt;

impl ChainExt for Scroll {
    type Data<Tr: ChainExt> = Never;

    type Fetch<Tr: ChainExt> = Never;

    type Aggregate<Tr: ChainExt> = Never;

    type MsgError = ();

    type Config = Never;
}
