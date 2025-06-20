use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::FromStr,
};

use bytes::Bytes;
use hex::decode;
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sha2::Digest;
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::debug;

use crate::indexer::{
    api::{BlockHeight, BlockRange, BlockReference, FetcherClient, IndexerError, UniversalChainId},
    nats::subject_for_block,
    postgres::nats::schedule,
    Indexer,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HubbleEvent {
    pub version: u8,
    pub universal_chain_id: UniversalChainId,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<Chunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<BlockEvents>,
}

impl HubbleEvent {
    pub fn events_by_height<'a>(&'a self) -> HashMap<BlockHeight, Vec<&'a BlockEvent>> {
        let mut by_height: HashMap<BlockHeight, Vec<&'a BlockEvent>> = HashMap::new();

        if let Some(es) = &self.events {
            for event in &es.events {
                by_height.entry(event.height()).or_default().push(event);
            }
        }

        by_height
    }
}

impl Display for HubbleEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} (events: {}, height: {})",
            self.universal_chain_id,
            self.range,
            self.events
                .as_ref()
                .map(|e| e.events.len().to_string())
                .unwrap_or("-".to_string()),
            self.events
                .as_ref()
                .map(|es| {
                    es.events
                        .iter()
                        .map(|e| e.height())
                        .sorted()
                        .unique()
                        .join(", ")
                })
                .unwrap_or_else(|| "-".to_string()),
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Range {
    #[serde(with = "flexible_u64")]
    pub start_inclusive: u64,
    #[serde(with = "flexible_u64")]
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
            // a range for a single block starts at the block height (inclusive) and ...
            start_inclusive: value.height,
            // ends one block after the block height (because it's exclusive).
            end_exclusive: value.height + 1,
        }
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{})", self.start_inclusive, self.end_exclusive)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
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

impl FromStr for MessageHash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let bytes = decode(s)?;
        Ok(MessageHash {
            message_hash: Bytes::from(bytes),
        })
    }
}

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
        .map_err(|e| IndexerError::InternalError(e.into()))?;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockEvents {
    pub events: Vec<BlockEvent>,
}

impl BlockEvents {
    pub fn new(events: Vec<BlockEvent>) -> Self {
        Self { events }
    }
}

#[warn(clippy::enum_variant_names)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum BlockEvent {
    // using database representation of fields. all these 'record representations' will
    // be replaced by USC events
    #[serde(rename = "ethereum-log")]
    EthereumLog {
        internal_chain_id: i32,
        block_hash: String,
        data: Value,
        #[serde(with = "flexible_u64")]
        height: BlockHeight,
        time: OffsetDateTime,
    },

    #[serde(rename = "tendermint-block")]
    TendermintBlock {
        internal_chain_id: i32,
        hash: String,
        data: Value,
        #[serde(with = "flexible_u64")]
        height: BlockHeight,
        time: OffsetDateTime,
    },

    #[serde(rename = "tendermint-transaction")]
    TendermintTransaction {
        internal_chain_id: i32,
        block_hash: String,
        #[serde(with = "flexible_u64")]
        height: BlockHeight,
        hash: String,
        data: Value,
        index: i32,
    },

    #[serde(rename = "tendermint-event")]
    TendermintEvent {
        internal_chain_id: i32,
        block_hash: String,
        #[serde(with = "flexible_u64")]
        height: BlockHeight,
        transaction_hash: Option<String>,
        index: i32,
        transaction_index: Option<i32>,
        data: Value,
        time: OffsetDateTime,
        flow: String,
    },
}

impl BlockEvent {
    pub fn height(&self) -> BlockHeight {
        match self {
            BlockEvent::TendermintBlock { height, .. } => *height,
            BlockEvent::TendermintTransaction { height, .. } => *height,
            BlockEvent::TendermintEvent { height, .. } => *height,
            BlockEvent::EthereumLog { height, .. } => *height,
        }
    }
}

mod flexible_u64 {
    use super::*;

    pub fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Number(n) => n
                .as_u64()
                .ok_or_else(|| serde::de::Error::custom("invalid number")),
            Value::String(s) => s.parse().map_err(serde::de::Error::custom),
            _ => Err(serde::de::Error::custom("expected number or string")),
        }
    }
}
