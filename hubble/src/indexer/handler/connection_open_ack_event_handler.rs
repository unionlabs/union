use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::connection_open_ack_event::ConnectionOpenAckEvent,
    handler::EventContext,
    record::{connection_open_ack_record::ConnectionOpenAckRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, ConnectionOpenAckEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ConnectionOpenAckRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
