use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{channel_open_init_event::ChannelOpenInitEvent, types::InternalChainIdContext},
    record::channel_open_init_record::ChannelOpenInitRecord,
};

impl<'a> InternalChainIdContext<'a, ChannelOpenInitEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ChannelOpenInitRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
