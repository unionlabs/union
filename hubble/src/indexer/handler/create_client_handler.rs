use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{create_client_event::CreateClientEvent, types::InternalChainIdContext},
    record::create_client_record::CreateClientRecord,
};

impl<'a> InternalChainIdContext<'a, CreateClientEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        CreateClientRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
