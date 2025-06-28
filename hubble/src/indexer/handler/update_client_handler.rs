use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::update_client_event::UpdateClientEvent,
    handler::EventContext,
    record::{update_client_record::UpdateClientRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, UpdateClientEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        UpdateClientRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
