use std::fmt::{self, Display};

use bytes::Bytes;
use serde::Serializer;
use serde_json::Value;
use sha2::Digest;
use sqlx::Postgres;
use tracing::debug;

use crate::indexer::{
    api::{BlockRange, BlockReference, FetcherClient, IndexerError},
    nats::subject_for_block,
    postgres::nats::schedule,
    Indexer,
};

#[derive(serde::Serialize)]
pub struct HubbleEvent {
    pub version: u8,
    pub universal_chain_id: String,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<Chunk>,
    #[serde(skip_serializing_if = "Value::is_null")]
    pub events: Value,
}

fn serialize_as_str<S, T>(x: &T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    s.serialize_str(&x.to_string())
}

#[derive(serde::Serialize, Clone)]
pub struct Range {
    #[serde(serialize_with = "serialize_as_str")]
    pub start_inclusive: u64,
    #[serde(serialize_with = "serialize_as_str")]
    pub end_exclusive: u64,
}

impl From<&BlockRange> for Range {
    fn from(value: &BlockRange) -> Self {
        Self {
            start_inclusive: value.start_inclusive,
            end_exclusive: value.end_exclusive,
        }
    }
}

impl From<&BlockReference> for Range {
    fn from(value: &BlockReference) -> Self {
        Self {
            start_inclusive: value.height,
            end_exclusive: value.height + 1,
        }
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{})", self.start_inclusive, self.end_exclusive)
    }
}

#[derive(serde::Serialize)]
pub struct Chunk {
    pub index: u8,
    pub total: u8,
}

#[derive(Clone, PartialEq, Eq)]
pub struct MessageHash {
    pub message_hash: bytes::Bytes,
}

impl MessageHash {
    fn new(message: &[u8]) -> Self {
        let mut hasher = sha2::Sha256::new();
        hasher.update(message);
        let event_hash = hasher.finalize();

        MessageHash {
            message_hash: Bytes::copy_from_slice(&event_hash),
        }
    }
}

impl From<Vec<u8>> for MessageHash {
    fn from(value: Vec<u8>) -> Self {
        Self {
            message_hash: value.into(),
        }
    }
}

impl From<MessageHash> for Vec<u8> {
    fn from(val: MessageHash) -> Self {
        val.message_hash.into()
    }
}

impl fmt::Display for MessageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.message_hash))
    }
}

impl<T: FetcherClient> Indexer<T> {
    pub async fn schedule_message(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        range: Range,
        events: Value,
    ) -> Result<MessageHash, IndexerError> {
        self.schedule_message_internal(tx, range, events, None)
            .await
    }

    pub async fn schedule_message_dedup(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        range: Range,
        events: Value,
        dedup_message_hash: &MessageHash,
    ) -> Result<MessageHash, IndexerError> {
        self.schedule_message_internal(tx, range, events, Some(dedup_message_hash))
            .await
    }

    pub async fn schedule_message_internal(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        range: Range,
        events: Value,
        dedup_message_hash: Option<&MessageHash>,
    ) -> Result<MessageHash, IndexerError> {
        let data = serde_json::to_vec(&HubbleEvent {
            version: 1,
            universal_chain_id: self.indexer_id.clone(),
            range: range.clone(),
            chunk: None,
            events,
        })
        .map_err(|e| IndexerError::InternalError(e.into()))?;

        let message_hash = MessageHash::new(&data);
        debug!("event_hash: {message_hash}");

        if dedup_message_hash == Some(&message_hash) {
            debug!("deduplicating: {}", range);
            return Ok(message_hash);
        }

        if self.nats.is_some() {
            debug!("scheduling: {}", range);

            let subject = subject_for_block(&self.indexer_id);

            let id = schedule(tx, &subject, data.into()).await?;

            debug!("scheduled: {range} - {id}");
        }

        Ok(message_hash)
    }
}
