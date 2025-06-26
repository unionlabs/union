use tracing::trace;

use crate::indexer::{
    api::IndexerError, event::channel_open_confirm_event::ChannelOpenConfirmEvent,
    record::channel_open_confirm_record::ChannelOpenConfirmRecord,
};

use crate::indexer::handler::EventContext;
use crate::indexer::record::ChainContext;
impl<'a> EventContext<'a, ChainContext, ChannelOpenConfirmEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ChannelOpenConfirmRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
