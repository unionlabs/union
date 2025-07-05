use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::channel_open_confirm_event::ChannelOpenConfirmEvent,
    handler::EventContext,
    record::{
        change_counter::Changes, channel_open_confirm_record::ChannelOpenConfirmRecord,
        ChainContext,
    },
};
impl<'a> EventContext<'a, ChainContext, ChannelOpenConfirmEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("handle({self:?})");

        ChannelOpenConfirmRecord::try_from(self)?.insert(tx).await
    }
}
