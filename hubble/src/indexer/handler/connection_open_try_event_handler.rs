use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::connection_open_try_event::ConnectionOpenTryEvent,
    handler::EventContext,
    record::{
        change_counter::Changes, connection_open_try_record::ConnectionOpenTryRecord, ChainContext,
    },
};
impl<'a> EventContext<'a, ChainContext, ConnectionOpenTryEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        ConnectionOpenTryRecord::try_from(self)?.insert(tx).await
    }
}
