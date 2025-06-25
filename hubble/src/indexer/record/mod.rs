use time::OffsetDateTime;

use crate::indexer::{
    api::IndexerError,
    event::types::{
        BlockHash, BlockHeight, BlockTimestamp, CanonicalChainId, ChannelId, ClientId, ClientType,
        ConnectionId, EventIndex, InternalChainId, MessageHash, MessageSequence,
        NatsConsumerSequence, NatsStreamSequence, PortId, TransactionEventIndex, TransactionHash,
        TransactionIndex, UniversalChainId, Version,
    },
};

pub(crate) mod channel_open_ack_record;
pub(crate) mod channel_open_confirm_record;
pub(crate) mod channel_open_init_record;
pub(crate) mod channel_open_try_record;
pub(crate) mod connection_open_ack_record;
pub(crate) mod connection_open_confirm_record;
pub(crate) mod connection_open_init_record;
pub(crate) mod connection_open_try_record;
pub(crate) mod create_client_record;
pub(crate) mod create_lens_client_record;
pub(crate) mod event_handler;
pub(crate) mod update_client_record;

impl UniversalChainId {
    pub fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}

impl CanonicalChainId {
    pub fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}

impl BlockHeight {
    pub fn pg_value(self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "block-height".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl ConnectionId {
    pub fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "connection-id".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl InternalChainId {
    pub fn pg_value(&self) -> Result<i32, IndexerError> {
        Ok(self.0)
    }
}
impl ClientType {
    pub fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl ChannelId {
    pub fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "channel-id".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl ClientId {
    pub fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "client-id".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PortId {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl Version {
    pub fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl BlockHash {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl MessageHash {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.message_hash.to_vec())
    }
}
impl TransactionHash {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl EventIndex {
    pub fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "event-index".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl MessageSequence {
    pub fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "message-sequence".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl NatsStreamSequence {
    pub fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "nats-stream-sequence".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl NatsConsumerSequence {
    pub fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "nats-consumer-sequence".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl TransactionIndex {
    pub fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "transaction-index".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl TransactionEventIndex {
    pub fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "transaction-event-index".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl BlockTimestamp {
    pub fn pg_value(&self) -> Result<OffsetDateTime, IndexerError> {
        Ok(self.0)
    }
}
