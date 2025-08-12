use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    enrich::enrich_create_wrapped_token_record,
    event::create_wrapped_token::CreateWrappedTokenEvent,
    handler::EventContext,
    record::{
        change_counter::Changes, create_wrapped_token_record::CreateWrappedTokenRecord,
        ChainContext,
    },
};
impl<'a> EventContext<'a, ChainContext, CreateWrappedTokenEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        let record = CreateWrappedTokenRecord::try_from(self)?;
        let mut changes = Changes::default();
        changes += record.insert(tx).await?;
        changes += enrich_create_wrapped_token_record(tx, record).await?;

        Ok(changes)
    }
}
