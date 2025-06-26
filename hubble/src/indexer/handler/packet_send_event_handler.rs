use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::packet_send_event::PacketSendEvent,
    handler::EventContext,
    record::{packet_send_record::PacketSendRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, PacketSendEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        PacketSendRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
