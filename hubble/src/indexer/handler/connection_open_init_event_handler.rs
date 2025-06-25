use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{connection_open_init_event::ConnectionOpenInitEvent, types::InternalChainIdContext},
    record::connection_open_init_record::ConnectionOpenInitRecord,
};

impl<'a> InternalChainIdContext<'a, ConnectionOpenInitEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ConnectionOpenInitRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
