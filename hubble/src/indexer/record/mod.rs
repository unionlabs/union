use std::fmt::{self, Display, Formatter};

use sqlx::types::BigDecimal;
use time::OffsetDateTime;

use crate::indexer::{
    api::IndexerError,
    event::types::{
        Acknowledgement, BlockHash, BlockHeight, BlockTimestamp, CanonicalChainId, Capacity,
        ChannelId, ClientId, ClientType, ConnectionId, ContractAddress, Denom, EventIndex, Maker,
        MakerMsg, MessageHash, MessageSequence, MutationAmount, MutationDirection,
        NatsConsumerSequence, NatsStreamSequence, PacketData, PacketHash, PortId, RefillRate,
        TimeoutTimestamp, TransactionEventIndex, TransactionHash, TransactionIndex,
        UniversalChainId, Version, WalletAddress,
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
pub(crate) mod token_bucket_update_record;
pub(crate) mod update_client_record;
pub(crate) mod wallet_mutation_entry_record;
pub(crate) mod write_ack_record;

pub trait PgValue<T, E = IndexerError> {
    fn pg_value(&self) -> Result<T, E>;
}

pub trait PgValueExt<T> {
    fn pg_value(&self) -> Result<Option<T>, IndexerError>;
}

impl<T, U> PgValueExt<U> for Option<T>
where
    T: PgValue<U>,
{
    fn pg_value(&self) -> Result<Option<U>, IndexerError> {
        match self {
            Some(v) => Ok(Some(v.pg_value()?)),
            None => Ok(None),
        }
    }
}

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

impl PgValue<String> for ChainNetwork {
    fn pg_value(&self) -> Result<String, IndexerError> {
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

impl PgValue<String> for UniversalChainId {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}

impl PgValue<String> for CanonicalChainId {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}

impl PgValue<i64> for BlockHeight {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "block-height-i64".to_string(),
                self.0.to_string(),
            )
        })
    }
}

// not implementing trait, because sqlx macro gets confused
// temporary, because we should remove numeric block height values
impl BlockHeight {
    fn pg_value_numeric(&self) -> Result<BigDecimal, IndexerError> {
        Ok(self.0.into())
    }
}

impl PgValue<i32> for ConnectionId {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "connection-id".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<i32> for InternalChainId {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        Ok(self.0)
    }
}
impl InternalChainId {
    /// temporary to support inconsistency in the datamodel: internal chain id should be `integer` type
    pub fn pg_value_numeric(&self) -> Result<BigDecimal, IndexerError> {
        Ok(self.0.into())
    }
}
impl PgValue<String> for ClientType {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl PgValue<i32> for ChannelId {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "channel-id".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<i32> for ClientId {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "client-id".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<Vec<u8>> for PortId {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<String> for Version {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl PgValue<Vec<u8>> for BlockHash {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for MessageHash {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.message_hash.to_vec())
    }
}
impl PgValue<Vec<u8>> for TransactionHash {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for PacketHash {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for PacketData {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for Acknowledgement {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for Maker {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for MakerMsg {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}

impl PgValue<i32> for EventIndex {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "event-index-i32".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<i64> for EventIndex {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "event-index-i64".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl PgValue<i64> for MessageSequence {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "message-sequence".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl PgValue<i64> for NatsStreamSequence {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "nats-stream-sequence".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl PgValue<i64> for NatsConsumerSequence {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "nats-consumer-sequence".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl PgValue<i64> for TransactionIndex {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "transaction-index".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<i64> for TransactionEventIndex {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "transaction-event-index".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<OffsetDateTime> for BlockTimestamp {
    fn pg_value(&self) -> Result<OffsetDateTime, IndexerError> {
        Ok(self.0)
    }
}
impl PgValue<BigDecimal> for TimeoutTimestamp {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}

impl PgValue<Vec<u8>> for Denom {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}

impl PgValue<BigDecimal> for Capacity {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}

impl PgValue<BigDecimal> for RefillRate {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}

impl PgValue<Vec<u8>> for ContractAddress {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for WalletAddress {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<BigDecimal> for MutationAmount {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}
impl PgValue<String> for MutationDirection {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(match self {
            MutationDirection::In => "in".to_string(),
            MutationDirection::Out => "out".to_string(),
        })
    }
}
