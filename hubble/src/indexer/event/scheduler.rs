use std::collections::HashMap;

use sqlx::Postgres;
use tracing::debug;

use crate::indexer::{
    api::{FetcherClient, IndexerError},
    event::{
        hubble::HubbleEvent,
        types::{BlockEvents, MessageHash, Range},
    },
    nats::subject_for_block,
    postgres::nats::schedule,
    Indexer,
};

impl<T: FetcherClient> Indexer<T> {
    pub async fn schedule_message(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        range: Range,
        events: Option<BlockEvents>,
    ) -> Result<MessageHash, IndexerError> {
        self.schedule_message_internal(tx, range, events, None)
            .await
    }

    pub async fn schedule_message_dedup(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        range: Range,
        events: Option<BlockEvents>,
        dedup_message_hash: &MessageHash,
    ) -> Result<MessageHash, IndexerError> {
        self.schedule_message_internal(tx, range, events, Some(dedup_message_hash))
            .await
    }

    pub async fn schedule_message_internal(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        range: Range,
        events: Option<BlockEvents>,
        dedup_message_hash: Option<&MessageHash>,
    ) -> Result<MessageHash, IndexerError> {
        let data = serde_json::to_vec(&HubbleEvent {
            version: 1,
            universal_chain_id: self.universal_chain_id.clone(),
            range: range.clone(),
            chunk: None,
            events,
        })
        .map_err(|e| IndexerError::InternalError(Box::new(e.into())))?;

        let message_hash = MessageHash::new(&data);
        debug!("event_hash: {message_hash}");

        if dedup_message_hash == Some(&message_hash) {
            debug!("deduplicating: {}", range);
            return Ok(message_hash);
        }

        let headers: HashMap<String, Vec<String>> =
            vec![("Message-Hash".to_string(), vec![message_hash.to_string()])]
                .into_iter()
                .collect();

        if self.nats.is_some() {
            debug!("scheduling: {}", range);

            let subject = subject_for_block(&self.universal_chain_id);

            let id = schedule(tx, &subject, data.into(), &headers).await?;

            debug!("scheduled: {range} - {id}");
        }

        Ok(message_hash)
    }
}
