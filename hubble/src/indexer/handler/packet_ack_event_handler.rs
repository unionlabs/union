use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::packet_ack_event::PacketAckEvent,
    handler::EventContext,
    record::{change_counter::Changes, packet_ack_record::PacketAckRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, PacketAckEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        PacketAckRecord::try_from(self)?.insert(tx).await
    }
}
