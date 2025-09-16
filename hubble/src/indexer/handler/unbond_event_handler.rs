use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::unbond_event::UnbondEvent,
    handler::EventContext,
    record::{change_counter::Changes, unbond_record::UnbondRecord, ChainContext},
};

impl<'a> EventContext<'a, ChainContext, UnbondEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        UnbondRecord::try_from(self)?.insert(tx).await
    }
}
