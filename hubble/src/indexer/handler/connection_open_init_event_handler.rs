use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::connection_open_init_event::ConnectionOpenInitEvent,
    handler::EventContext,
    record::{
        change_counter::Changes, connection_open_init_record::ConnectionOpenInitRecord,
        ChainContext,
    },
};
impl<'a> EventContext<'a, ChainContext, ConnectionOpenInitEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        ConnectionOpenInitRecord::try_from(self)?.insert(tx).await
    }
}
