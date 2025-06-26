use std::fmt::{self, Display, Formatter};

use sqlx::types::BigDecimal;
use time::OffsetDateTime;

use crate::indexer::{
    api::IndexerError,
    event::types::{
        Acknowledgement, BlockHash, BlockHeight, BlockTimestamp, CanonicalChainId, ChannelId,
        ClientId, ClientType, ConnectionId, EventIndex, Maker, MakerMsg, MessageHash,
        MessageSequence, NatsConsumerSequence, NatsStreamSequence, PacketData, PacketHash, PortId,
        TimeoutTimestamp, TransactionEventIndex, TransactionHash, TransactionIndex,
        UniversalChainId, Version,
    },
    handler::EventContext,
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
pub(crate) mod packet_ack_record;
pub(crate) mod packet_recv_record;
pub(crate) mod packet_send_record;
pub(crate) mod packet_timeout_record;
pub(crate) mod update_client_record;
pub(crate) mod write_ack_record;

/// wrapper required until we've migrated to use universal-chain-ids
pub struct ChainContext {
    pub internal_chain_id: InternalChainId,
    pub network: ChainNetwork,
}

impl Display for ChainContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[chain={}, network={}]",
            self.internal_chain_id, self.network
        )
    }
}

impl std::fmt::Debug for ChainContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InternalChainIdContext")
            .field("internal_chain_id", &self.internal_chain_id)
            .field("network", &self.network)
            .finish()
    }
}

impl ChainContext {
    pub fn with_event<'a, E>(&'a self, event: &'a E) -> EventContext<'a, ChainContext, E> {
        EventContext {
            context: self,
            event,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
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

#[derive(Debug)]
pub enum ChainNetwork {
    Mainnet,
    Testnet,
}

impl ChainNetwork {
    pub fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(match self {
            ChainNetwork::Mainnet => "mainnet".to_string(),
            ChainNetwork::Testnet => "testnet".to_string(),
        })
    }
}

impl Display for ChainNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ChainNetwork::Mainnet => "mainnet".to_string(),
                ChainNetwork::Testnet => "testnet".to_string(),
            }
        )
    }
}

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
    pub fn pg_value_bigint(self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "block-height".to_string(),
                self.0.to_string(),
            )
        })
    }

    pub fn pg_value_numeric(self) -> Result<BigDecimal, IndexerError> {
        Ok(self.0.into())
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
    pub fn pg_value_integer(&self) -> Result<i32, IndexerError> {
        Ok(self.0)
    }
    pub fn pg_value_numeric(&self) -> Result<BigDecimal, IndexerError> {
        Ok(self.0.into())
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
impl PacketHash {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PacketData {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl Acknowledgement {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl Maker {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl MakerMsg {
    pub fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl EventIndex {
    pub fn pg_value_bigint(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "event-index-i64".to_string(),
                self.0.to_string(),
            )
        })
    }
    pub fn pg_value_integer(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "event-index-i32".to_string(),
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
impl TimeoutTimestamp {
    pub fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}
