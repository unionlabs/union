use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{connection_open_try_event::ConnectionOpenTryEvent, types::InternalChainIdContext},
    record::connection_open_try_record::ConnectionOpenTryRecord,
};

impl<'a> InternalChainIdContext<'a, ConnectionOpenTryEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ConnectionOpenTryRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
