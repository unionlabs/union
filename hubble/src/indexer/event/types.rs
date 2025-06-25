use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use bytes::Bytes;
use hex::decode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sha2::Digest;
use time::OffsetDateTime;

use crate::indexer::{api::IndexerError, event::supported::SupportedBlockEvent};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Range {
    #[serde(with = "flexible_u64")]
    pub start_inclusive: u64,
    #[serde(with = "flexible_u64")]
    pub end_exclusive: u64,
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
    pub fn new(message: &[u8]) -> Self {
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
#[derive(Serialize, Deserialize, Debug)]
pub struct BlockEvents {
    pub events: Vec<BlockEvent>,
}

impl BlockEvents {
    pub fn new(events: Vec<BlockEvent>) -> Self {
        Self { events }
    }
}

impl FromIterator<BlockEvent> for BlockEvents {
    fn from_iter<I: IntoIterator<Item = BlockEvent>>(iter: I) -> Self {
        BlockEvents {
            events: iter.into_iter().collect(),
        }
    }
}

impl From<Vec<SupportedBlockEvent>> for BlockEvents {
    fn from(events: Vec<SupportedBlockEvent>) -> Self {
        events.into_iter().map(Into::into).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum BlockEvent {
    Supported(SupportedBlockEvent),
    Unsupported(UnsupportedBlockEvent),
}

impl From<SupportedBlockEvent> for BlockEvent {
    fn from(value: SupportedBlockEvent) -> Self {
        BlockEvent::Supported(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct UniversalChainId(pub String);

impl From<String> for UniversalChainId {
    fn from(value: String) -> Self {
        UniversalChainId(value)
    }
}

impl Display for UniversalChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord, Copy)]
pub struct BlockHeight(#[serde(with = "flexible_u64")] pub u64);

impl TryFrom<i64> for BlockHeight {
    type Error = IndexerError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(BlockHeight(u64::try_from(value).map_err(|_| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "block-height".to_string(),
                value.to_string(),
            )
        })?))
    }
}

impl From<u64> for BlockHeight {
    fn from(value: u64) -> Self {
        BlockHeight(value)
    }
}

impl Display for BlockHeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord, Copy)]
pub struct MessageSequence(#[serde(with = "flexible_u64")] pub u64);

impl TryFrom<i64> for MessageSequence {
    type Error = IndexerError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(MessageSequence(u64::try_from(value).map_err(|_| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "message-sequence".to_string(),
                value.to_string(),
            )
        })?))
    }
}

impl From<u64> for MessageSequence {
    fn from(value: u64) -> Self {
        MessageSequence(value)
    }
}

impl Display for MessageSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord, Copy)]
pub struct NatsStreamSequence(#[serde(with = "flexible_u64")] pub u64);

impl TryFrom<i64> for NatsStreamSequence {
    type Error = IndexerError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(NatsStreamSequence(u64::try_from(value).map_err(|_| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "nats-stream-sequence".to_string(),
                value.to_string(),
            )
        })?))
    }
}

impl From<u64> for NatsStreamSequence {
    fn from(value: u64) -> Self {
        NatsStreamSequence(value)
    }
}

impl Display for NatsStreamSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "s{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord, Copy)]
pub struct NatsConsumerSequence(#[serde(with = "flexible_u64")] pub u64);

impl TryFrom<i64> for NatsConsumerSequence {
    type Error = IndexerError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(NatsConsumerSequence(u64::try_from(value).map_err(
            |_| {
                IndexerError::InternalCannotMapFromDatabaseDomain(
                    "nats-consumer-sequence".to_string(),
                    value.to_string(),
                )
            },
        )?))
    }
}

impl From<u64> for NatsConsumerSequence {
    fn from(value: u64) -> Self {
        NatsConsumerSequence(value)
    }
}

impl Display for NatsConsumerSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionId(pub u32);

impl From<u32> for ConnectionId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct InternalChainId(pub i32);

impl From<i32> for InternalChainId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl Display for InternalChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// wrapper required until we've migrated to use universal-chain-ids
pub struct InternalChainIdContext<'a, T> {
    pub internal_chain_id: InternalChainId,
    pub event: &'a T,
}

impl<'a, T> InternalChainIdContext<'a, T> {
    pub fn new(internal_chain_id: InternalChainId, event: &'a T) -> Self {
        InternalChainIdContext {
            internal_chain_id,
            event,
        }
    }
}

impl<'a, T> Display for InternalChainIdContext<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[chain={}] {}", self.internal_chain_id, self.event)
    }
}

impl<'a, T> std::fmt::Debug for InternalChainIdContext<'a, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InternalChainIdContext")
            .field("internal_chain_id", &self.internal_chain_id)
            .field("event", &self.event)
            .finish()
    }
}

impl InternalChainId {
    pub fn context<'a, T>(self, event: &'a T) -> InternalChainIdContext<'a, T> {
        InternalChainIdContext::new(self, event)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelId(pub u32);

impl From<u32> for ChannelId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortId(pub bytes::Bytes);

impl From<bytes::Bytes> for PortId {
    fn from(value: bytes::Bytes) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version(pub String);

impl From<String> for Version {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHash(pub bytes::Bytes);

impl From<bytes::Bytes> for BlockHash {
    fn from(value: bytes::Bytes) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionHash(pub bytes::Bytes);

impl From<bytes::Bytes> for TransactionHash {
    fn from(value: bytes::Bytes) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventIndex(#[serde(with = "flexible_u64")] pub u64);

impl From<u64> for EventIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionIndex(#[serde(with = "flexible_u64")] pub u64);

impl From<u64> for TransactionIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionEventIndex(#[serde(with = "flexible_u64")] pub u64);

impl TryFrom<usize> for TransactionEventIndex {
    type Error = IndexerError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(u64::try_from(value).map_err(|_| {
            IndexerError::CannotMapToEventDomain(
                "transaction-event-index".to_string(),
                value.to_string(),
            )
        })?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockTimestamp(pub OffsetDateTime);

impl From<OffsetDateTime> for BlockTimestamp {
    fn from(value: OffsetDateTime) -> Self {
        Self(value)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsupportedBlockEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub raw: serde_json::Value,
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
