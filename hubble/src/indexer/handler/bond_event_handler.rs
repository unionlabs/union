use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::bond_event::BondEvent,
    handler::EventContext,
    record::{bond_record::BondRecord, change_counter::Changes, ChainContext},
};

impl<'a> EventContext<'a, ChainContext, BondEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        BondRecord::try_from(self)?.insert(tx).await
    }
}
