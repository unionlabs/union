use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::token_bucket_update_event::TokenBucketUpdateEvent,
    handler::EventContext,
    record::{token_bucket_update_record::TokenBucketUpdateRecord, ChainContext},
};
impl<'a> EventContext<'a, ChainContext, TokenBucketUpdateEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        TokenBucketUpdateRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
