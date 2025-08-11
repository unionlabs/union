use tracing::trace;

use crate::indexer::{
    api::IndexerError,
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

        CreateWrappedTokenRecord::try_from(self)?.insert(tx).await
    }
}
