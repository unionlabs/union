use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    enrich::enrich,
    event::packet_send_event::PacketSendEvent,
    handler::EventContext,
    record::{change_counter::Changes, packet_send_record::PacketSendRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, PacketSendEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        let record = PacketSendRecord::try_from(self)?;
        let mut changes = Changes::default();
        changes += record.insert(tx).await?;
        changes += enrich(tx, record).await?;

        Ok(changes)
    }
}
