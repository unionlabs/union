use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    enrich::enrich,
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

        let record = PacketSendRecord::try_from(self)?;
        record.insert(tx).await?;

        enrich(tx, record).await
    }
}
