use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::channel_open_init_event::ChannelOpenInitEvent,
    handler::EventContext,
    record::{
        change_counter::Changes, channel_open_init_record::ChannelOpenInitRecord, ChainContext,
    },
};
impl<'a> EventContext<'a, ChainContext, ChannelOpenInitEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        ChannelOpenInitRecord::try_from(self)?.insert(tx).await
    }
}
