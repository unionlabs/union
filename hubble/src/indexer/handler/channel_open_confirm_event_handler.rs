use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{channel_open_confirm_event::ChannelOpenConfirmEvent, types::InternalChainIdContext},
    record::channel_open_confirm_record::ChannelOpenConfirmRecord,
};

impl<'a> InternalChainIdContext<'a, ChannelOpenConfirmEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ChannelOpenConfirmRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
