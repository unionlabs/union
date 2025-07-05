use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::packet_timeout_event::PacketTimeoutEvent,
    handler::EventContext,
    record::{change_counter::Changes, packet_timeout_record::PacketTimeoutRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, PacketTimeoutEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        PacketTimeoutRecord::try_from(self)?.insert(tx).await
    }
}
