use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{connection_open_ack_event::ConnectionOpenAckEvent, types::InternalChainIdContext},
    record::connection_open_ack_record::ConnectionOpenAckRecord,
};

impl<'a> InternalChainIdContext<'a, ConnectionOpenAckEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ConnectionOpenAckRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
