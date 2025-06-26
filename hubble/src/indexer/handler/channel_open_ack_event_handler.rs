use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::channel_open_ack_event::ChannelOpenAckEvent,
    handler::EventContext,
    record::{channel_open_ack_record::ChannelOpenAckRecord, ChainContext},
};

impl<'a> EventContext<'a, ChainContext, ChannelOpenAckEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ChannelOpenAckRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
