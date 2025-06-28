use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::create_lens_client_event::CreateLensClientEvent,
    handler::EventContext,
    record::{create_lens_client_record::CreateLensClientRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, CreateLensClientEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        CreateLensClientRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
