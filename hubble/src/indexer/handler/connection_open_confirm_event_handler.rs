use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{
        connection_open_confirm_event::ConnectionOpenConfirmEvent, types::InternalChainIdContext,
    },
    record::connection_open_confirm_record::ConnectionOpenConfirmRecord,
};

impl<'a> InternalChainIdContext<'a, ConnectionOpenConfirmEvent> {
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
