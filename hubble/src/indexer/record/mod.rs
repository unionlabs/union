use std::fmt::{self, Display, Formatter};

use sqlx::types::BigDecimal;
use time::OffsetDateTime;

use crate::indexer::{
    api::IndexerError,
    event::types::{
        Acknowledgement, Batch, BlockHash, BlockHeight, BlockTimestamp, BondInAmount,
        BondMintAmount, BondMintToAddress, BondSenderAddress, CanonicalChainId, Capacity,
        ChannelId, ChannelVersion, ClientId, ClientType, ConnectionId, ContractAddress, Denom,
        EventIndex, Maker, MakerMsg, MessageHash, MessageIndex, MessageSequence, MutationAmount,
        MutationDirection, NatsConsumerSequence, NatsStreamSequence, Owner, PacketData, PacketHash,
        Path, PortId, ProxyAccount, RefillRate, TimeoutTimestamp, TransactionEventIndex,
        TransactionHash, TransactionIndex, UnbondAmount, UnbondIsNewRequest, UnbondStakerAddress,
        UniversalChainId, WalletAddress,
    },
    handler::{
        types::{
            AddressCanonical, AddressDisplay, AddressZkgm, Amount, CreateWrappedTokenKind, Fee,
            InstructionHash, InstructionIndex, InstructionOpcode, InstructionPath,
            InstructionRootPath, InstructionRootSalt, InstructionType, InstructionVersion,
            Metadata, OperandContractAddress, OperandSender, PacketShape, RpcType, TokenDecimals,
            TokenName, TokenOrderKind, TokenPath, TokenSymbol, TransferIndex, WrapDirection,
        },
        EventContext,
    },
};

pub(crate) mod bond_record;
pub(crate) mod change_counter;
pub(crate) mod channel_meta_data;
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
pub(crate) mod create_proxy_account_record;
pub(crate) mod create_wrapped_token_record;
pub(crate) mod create_wrapped_token_relation_record;
pub(crate) mod event_handler;
pub(crate) mod packet_ack_record;
pub(crate) mod packet_recv_record;
pub(crate) mod packet_send_bond_record;
pub(crate) mod packet_send_decoded_record;
pub(crate) mod packet_send_instructions_search_record;
pub(crate) mod packet_send_record;
pub(crate) mod packet_send_transfers_record;
pub(crate) mod packet_send_unbond_record;
pub(crate) mod packet_timeout_record;
pub(crate) mod token_bucket_update_record;
pub(crate) mod unbond_record;
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

impl PgValue<String> for RpcType {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(match self {
            RpcType::Cosmos => "cosmos",
            RpcType::Evm => "evm",
        }
        .to_string())
    }
}

#[derive(Debug, Clone, Copy)]
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

impl From<bool> for ChainNetwork {
    fn from(value: bool) -> Self {
        match value {
            // value is the 'testnet' property on chains table
            true => ChainNetwork::Testnet,
            false => ChainNetwork::Mainnet,
        }
    }
}

impl TryFrom<Option<bool>> for ChainNetwork {
    type Error = IndexerError;

    fn try_from(value: Option<bool>) -> Result<Self, Self::Error> {
        value.map(|value| value.into()).ok_or_else(|| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "chain-network".to_string(),
                "expecting Some, but got None".to_string(),
            )
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

impl TryFrom<(Option<String>, Option<String>)> for UniversalChainId {
    type Error = IndexerError;

    fn try_from(value: (Option<String>, Option<String>)) -> Result<Self, Self::Error> {
        match value {
            (Some(family), Some(chain_id)) => Ok(UniversalChainId(format!("{family}.{chain_id}"))),
            (Some(family), None) => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "universal-chain-id".to_string(),
                format!("expecting Some chain-id, but got None (family: {family}"),
            )),
            (None, Some(chain_id)) => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "universal-chain-id".to_string(),
                format!("expecting Some family, but got None (chain-id: {chain_id}"),
            )),
            (None, None) => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "universal-chain-id".to_string(),
                "expecting Some family and chain-id, but got None".to_string(),
            )),
        }
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

impl TryFrom<i32> for ConnectionId {
    type Error = IndexerError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        u32::try_from(value).map(|value| value.into()).map_err(|_| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "connection-id".to_string(),
                "expecting fits in u32, but got {value}".to_string(),
            )
        })
    }
}

impl TryFrom<Option<i32>> for ConnectionId {
    type Error = IndexerError;

    fn try_from(value: Option<i32>) -> Result<Self, Self::Error> {
        match value {
            Some(value) => value.try_into(),
            None => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "connection-id".to_string(),
                "expecting Some, but got None".to_string(),
            )),
        }
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
impl TryFrom<Option<String>> for ClientType {
    type Error = IndexerError;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        value.map(|value| value.into()).ok_or_else(|| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "client-type".to_string(),
                "expecting Some, but got None".to_string(),
            )
        })
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

impl TryFrom<i32> for ChannelId {
    type Error = IndexerError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        u32::try_from(value).map(|value| value.into()).map_err(|_| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "client-id".to_string(),
                "expecting fits in u32, but got {value}".to_string(),
            )
        })
    }
}

impl TryFrom<Option<i32>> for ChannelId {
    type Error = IndexerError;

    fn try_from(value: Option<i32>) -> Result<Self, Self::Error> {
        match value {
            Some(value) => value.try_into(),
            None => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "channel-id".to_string(),
                "expecting Some, but got None".to_string(),
            )),
        }
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

impl TryFrom<i32> for ClientId {
    type Error = IndexerError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        u32::try_from(value).map(|value| value.into()).map_err(|_| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "client-id".to_string(),
                "expecting fits in u32, but got {value}".to_string(),
            )
        })
    }
}

impl TryFrom<Option<i32>> for ClientId {
    type Error = IndexerError;

    fn try_from(value: Option<i32>) -> Result<Self, Self::Error> {
        match value {
            Some(value) => value.try_into(),
            None => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "client-id".to_string(),
                "expecting Some, but got None".to_string(),
            )),
        }
    }
}

impl PgValue<Vec<u8>> for PortId {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl TryFrom<Option<Vec<u8>>> for PortId {
    type Error = IndexerError;

    fn try_from(value: Option<Vec<u8>>) -> Result<Self, Self::Error> {
        value
            .map(|value| bytes::Bytes::from(value).into())
            .ok_or_else(|| {
                IndexerError::InternalCannotMapFromDatabaseDomain(
                    "port-id".to_string(),
                    "expecting Some, but got None".to_string(),
                )
            })
    }
}
impl PgValue<String> for ChannelVersion {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl TryFrom<Option<String>> for ChannelVersion {
    type Error = IndexerError;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        value.map(|value| value.into()).ok_or_else(|| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "channel-version".to_string(),
                "expecting Some, but got None".to_string(),
            )
        })
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
impl PgValue<BigDecimal> for Path {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
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
impl PgValue<i64> for MessageIndex {
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

impl PgValue<Vec<u8>> for Owner {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}

impl PgValue<Vec<u8>> for ProxyAccount {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}

impl PgValue<Vec<u8>> for Metadata {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}

impl PgValue<i32> for CreateWrappedTokenKind {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        Ok(u8::from(self.clone()).into())
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
impl PgValue<BigDecimal> for BondInAmount {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}
impl PgValue<BigDecimal> for BondMintAmount {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}
impl PgValue<Vec<u8>> for BondMintToAddress {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for BondSenderAddress {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<BigDecimal> for UnbondAmount {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}
impl PgValue<i64> for Batch {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "batch-i64".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<bool> for UnbondIsNewRequest {
    fn pg_value(&self) -> Result<bool, IndexerError> {
        Ok(self.0)
    }
}
impl PgValue<Vec<u8>> for UnbondStakerAddress {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
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

impl PgValue<i32> for TransferIndex {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "transfer-index".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<Vec<u8>> for AddressZkgm {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for AddressCanonical {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<String> for AddressDisplay {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl PgValue<BigDecimal> for Amount {
    fn pg_value(&self) -> Result<BigDecimal, IndexerError> {
        Ok(BigDecimal::new(self.0.into(), 0))
    }
}
impl PgValue<String> for TokenName {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl PgValue<Vec<u8>> for TokenPath {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<String> for TokenSymbol {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl PgValue<i32> for TokenDecimals {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        i32::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "token-decimals".to_string(),
                self.0.to_string(),
            )
        })
    }
}
impl PgValue<String> for Fee {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(match self {
            Fee::Instruction(..) => "instruction",
            Fee::QuoteDelta(..) => "quote_delta",
            Fee::QuoteDeltaNegative => "quote_delta_negative",
            Fee::None => "none",
            Fee::Swap => "swap",
        }
        .to_string())
    }
}

impl Fee {
    pub fn amount(&self) -> Option<Amount> {
        match self {
            Fee::Instruction(_, amount, _) => Some(amount.clone()),
            Fee::QuoteDelta(_, amount, _) => Some(amount.clone()),
            Fee::QuoteDeltaNegative => None,
            Fee::None => None,
            Fee::Swap => None,
        }
    }
}

impl Fee {
    pub fn token(&self) -> Option<Denom> {
        match self {
            Fee::Instruction(denom, _, _) => Some(denom.clone()),
            Fee::QuoteDelta(denom, _, _) => Some(denom.clone()),
            Fee::QuoteDeltaNegative => None,
            Fee::None => None,
            Fee::Swap => None,
        }
    }
}

impl PgValue<String> for WrapDirection {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(match self {
            WrapDirection::Wrapping => "wrapping",
            WrapDirection::Unwrapping => "unwrapping",
        }
        .to_string())
    }
}
impl PgValue<String> for PacketShape {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(match self {
            PacketShape::BatchV0TransferV0Fee => "batch_v0_transfer_v0_fee",
            PacketShape::TransferV0 => "transfer_v0",
            PacketShape::BatchV0TransferV1 => "batch_v0_transfer_v1",
            PacketShape::BatchV0TransferV1Fee => "batch_v0_transfer_v1_fee",
            PacketShape::TransferV1 => "transfer_v1",
            PacketShape::BatchV0TransferV2 => "batch_v0_transfer_v2",
            PacketShape::BatchV0TransferV2Fee => "batch_v0_transfer_v2_fee",
            PacketShape::TransferV2 => "transfer_v2",
            PacketShape::BondV2 => "bond_v2",
            PacketShape::UnbondV2 => "unbond_v2",
        }
        .to_string())
    }
}
impl PgValue<i64> for InstructionIndex {
    fn pg_value(&self) -> Result<i64, IndexerError> {
        i64::try_from(self.0).map_err(|_| {
            IndexerError::InternalCannotMapToDatabaseDomain(
                "instruction-index".to_string(),
                self.0.to_string(),
            )
        })
    }
}

impl PgValue<Vec<u8>> for InstructionHash {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}

impl PgValue<String> for InstructionType {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}

impl PgValue<Vec<u8>> for InstructionRootPath {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<Vec<u8>> for InstructionRootSalt {
    fn pg_value(&self) -> Result<Vec<u8>, IndexerError> {
        Ok(self.0.to_vec())
    }
}
impl PgValue<String> for InstructionPath {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(self.0.clone())
    }
}
impl PgValue<i32> for InstructionVersion {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        Ok(i32::from(self.0))
    }
}
impl PgValue<i32> for InstructionOpcode {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        Ok(i32::from(self.0))
    }
}
impl PgValue<i32> for TokenOrderKind {
    fn pg_value(&self) -> Result<i32, IndexerError> {
        Ok(u8::from(self.clone()).into())
    }
}
// currently stored as text, could be bytea
impl PgValue<String> for OperandSender {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(format!("0x{}", hex::encode(&self.0)))
    }
}
// currently stored as text, could be bytea
impl PgValue<String> for OperandContractAddress {
    fn pg_value(&self) -> Result<String, IndexerError> {
        Ok(format!("0x{}", hex::encode(&self.0)))
    }
}
