use enumorph::Enumorph;
use macros::apply;
use queue_msg::{call, data, defer_absolute, now, queue_msg, seq, Op, QueueError, SubsetOf};
use tracing::debug;
use unionlabs::{
    ibc::core::client::height::Height, ics24::ClientStatePath, id::ClientId, traits::Member,
    QueryHeight,
};

use crate::{
    data::LatestHeight,
    call::FetchState,
    json_rpc_error_to_queue_error,
    plugin::{ChainModuleClient, ClientModuleClient},
    top_level_identifiable_enum, Context, VoyagerMessage,
};

impl<D: Member, F: Member, A: Member> HandleWait<VoyagerMessage<D, F, A>> for Wait {
    // #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn handle(self, ctx: &Context) -> Result<Op<VoyagerMessage<D, F, A>>, QueueError> {
        match self {}
    }
}
