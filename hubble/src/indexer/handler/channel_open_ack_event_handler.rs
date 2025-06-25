use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{channel_open_ack_event::ChannelOpenAckEvent, types::InternalChainIdContext},
    record::channel_open_ack_record::ChannelOpenAckRecord,
};

impl<'a> InternalChainIdContext<'a, ChannelOpenAckEvent> {
    pub async fn handle(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), IndexerError> {
        trace!("handle({self:?})");

        ChannelOpenAckRecord::try_from(self)?.insert(tx).await?;

        Ok(())
    }
}
