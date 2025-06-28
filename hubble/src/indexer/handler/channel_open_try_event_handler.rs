use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::channel_open_try_event::ChannelOpenTryEvent,
    handler::EventContext,
    record::{channel_open_try_record::ChannelOpenTryRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, ChannelOpenTryEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ChannelOpenTryRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
