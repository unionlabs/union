use std::fmt;

use bytes::Bytes;
use serde::Serializer;
use serde_json::Value;
use sha2::Digest;
use sqlx::Postgres;
use tracing::debug;

use crate::indexer::{
    api::{BlockReference, FetcherClient, IndexerError},
    nats::subject_for_block,
    postgres::schedule,
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

#[derive(serde::Serialize)]
pub struct Range {
    #[serde(serialize_with = "serialize_as_str")]
    pub start: u64,
    #[serde(serialize_with = "serialize_as_str")]
    pub end: u64,
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
    pub async fn schedule_event(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        reference: &BlockReference,
        events: Value,
    ) -> Result<MessageHash, IndexerError> {
        self.schedule_event_internal(tx, reference, events, None)
            .await
    }

    pub async fn schedule_event_dedup(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        reference: &BlockReference,
        events: Value,
        dedup_event_hash: &MessageHash,
    ) -> Result<MessageHash, IndexerError> {
        self.schedule_event_internal(tx, reference, events, Some(dedup_event_hash))
            .await
    }

    pub async fn schedule_event_internal(
        &self,
        tx: &mut sqlx::Transaction<'_, Postgres>,
        reference: &BlockReference,
        events: Value,
        dedup_event_hash: Option<&MessageHash>,
    ) -> Result<MessageHash, IndexerError> {
        let data = serde_json::to_vec(&HubbleEvent {
            version: 1,
            universal_chain_id: self.indexer_id.clone(),
            range: Range {
                start: reference.height,
                end: reference.height,
            },
            chunk: None,
            events,
        })
        .map_err(|e| IndexerError::InternalError(e.into()))?;

        let event_hash = MessageHash::new(&data);
        debug!("event_hash: {event_hash}");

        if dedup_event_hash == Some(&event_hash) {
            debug!("deduplicating: {reference}");
            return Ok(event_hash);
        }

        if self.nats.is_some() {
            debug!("scheduling: {reference}");

            let subject = subject_for_block(&self.indexer_id);

            let id = schedule(tx, &subject, data.into()).await?;

            debug!("scheduled: {reference} - {id}");
        }

        Ok(event_hash)
    }
}
