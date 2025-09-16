use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::create_proxy_account_event::CreateProxyAccountEvent,
    handler::EventContext,
    record::{
        change_counter::Changes, create_proxy_account_record::CreateProxyAccountRecord,
        ChainContext,
    },
};
impl<'a> EventContext<'a, ChainContext, CreateProxyAccountEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        CreateProxyAccountRecord::try_from(self)?.insert(tx).await
    }
}
