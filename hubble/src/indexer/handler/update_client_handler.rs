use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{types::InternalChainIdContext, update_client_event::UpdateClientEvent},
    record::update_client_record::UpdateClientRecord,
};

impl<'a> InternalChainIdContext<'a, UpdateClientEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        UpdateClientRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
