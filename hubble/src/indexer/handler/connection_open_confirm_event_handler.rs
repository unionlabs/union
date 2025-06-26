use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::connection_open_confirm_event::ConnectionOpenConfirmEvent,
    handler::EventContext,
    record::{connection_open_confirm_record::ConnectionOpenConfirmRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, ConnectionOpenConfirmEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ConnectionOpenConfirmRecord::try_from(self)?
            .insert(tx)
            .await?;

        Ok(())
    }
}
