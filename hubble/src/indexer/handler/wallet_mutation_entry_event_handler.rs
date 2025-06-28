use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::wallet_mutation_entry_event::WalletMutationEntryEvent,
    handler::EventContext,
    record::{wallet_mutation_entry_record::WalletMutationEntryRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, WalletMutationEntryEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        WalletMutationEntryRecord::try_from(self)?
            .insert(tx)
            .await?;

        Ok(())
    }
}
