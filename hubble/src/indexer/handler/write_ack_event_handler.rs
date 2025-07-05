use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::write_ack_event::WriteAckEvent,
    handler::EventContext,
    record::{change_counter::Changes, write_ack_record::WriteAckRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, WriteAckEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        WriteAckRecord::try_from(self)?.insert(tx).await
    }
}
