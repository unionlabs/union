use std::{fmt::Display, ops::Deref};

use base64::{engine::general_purpose, Engine};
use bytes::Bytes;
use hex::encode;
use ruint::aliases::U256;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::indexer::{
    api::IndexerError,
    event::types::{
        CanonicalChainId, ChannelId, ChannelVersion, ClientId, ClientType, ConnectionId, Denom,
        PacketHash, PortId, UniversalChainId,
    },
    record::{ChainNetwork, InternalChainId},
};

#[derive(Clone)]
pub struct ChannelMetaData {
    /// deprecated; should be derived from universal_chain_id
    pub rpc_type: RpcType,
    /// deprecated; should be derived from universal_counterparty_chain_id
    pub counterparty_rpc_type: RpcType,
    pub internal_chain_id: InternalChainId,
    pub internal_counterparty_chain_id: InternalChainId,
    pub universal_chain_id: UniversalChainId,
    pub universal_counterparty_chain_id: UniversalChainId,
    pub canonical_chain_id: CanonicalChainId,
    pub canonical_counterparty_chain_id: CanonicalChainId,
    pub network: ChainNetwork,
    pub counterparty_network: ChainNetwork,
    pub client_type: ClientType,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub channel_id: ChannelId,
    pub counterparty_channel_id: ChannelId,
    pub port_id: PortId,
    pub counterparty_port_id: PortId,
    pub channel_version: ChannelVersion,
}

impl Display for ChannelMetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.channel_id)
    }
}

pub struct Transfer {
    pub transfer_index: TransferIndex,
    pub sender_zkgm: AddressZkgm,
    pub sender_canonical: AddressCanonical,
    pub sender_display: AddressDisplay,
    pub receiver_zkgm: AddressZkgm,
    pub receiver_canonical: AddressCanonical,
    pub receiver_display: AddressDisplay,
    pub base_token: Denom,
    pub base_amount: Amount,
    pub base_token_name: Option<TokenName>,
    pub base_token_path: Option<TokenPath>,
    pub base_token_symbol: Option<TokenSymbol>,
    pub base_token_decimals: Option<TokenDecimals>, // only None in transfer v0
    pub quote_token: Denom,
    pub quote_amount: Amount,
    pub kind: Option<TokenOrderKind>,
    pub metadata: Option<Metadata>,
    pub fee: Fee,
    pub wrap_direction: Option<WrapDirection>,
    pub packet_shape: PacketShape,
}

pub struct Bond {
    pub source_channel_id: ChannelId,
    pub universal_chain_id: UniversalChainId,
    pub remote_source_channel_id: ChannelId,
    pub remote_destination_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub source_client_id: ClientId,
    pub remote_source_client_id: ClientId,
    pub remote_destination_client_id: ClientId,
    pub destination_client_id: ClientId,
    pub source_connection_id: ConnectionId,
    pub remote_source_connection_id: ConnectionId,
    pub remote_destination_connection_id: ConnectionId,
    pub destination_connection_id: ConnectionId,
    pub source_port_id: PortId,
    pub remote_source_port_id: PortId,
    pub remote_destination_port_id: PortId,
    pub destination_port_id: PortId,
    pub internal_remote_chain_id: InternalChainId,
    pub internal_destination_chain_id: InternalChainId,
    pub remote_universal_chain_id: UniversalChainId,
    pub destination_universal_chain_id: UniversalChainId,
    pub source_network: ChainNetwork,
    pub remote_network: ChainNetwork,
    pub destination_network: ChainNetwork,
    pub sender_zkgm: AddressZkgm,
    pub sender_canonical: AddressCanonical,
    pub sender_display: AddressDisplay,
    pub receiver_zkgm: AddressZkgm, // (from send-call.receiver)
    pub receiver_canonical: AddressCanonical, // (from send-call.receiver)
    pub receiver_display: AddressDisplay, // (from send-call.receiver)
    pub base_token: Denom,          // 0xba5ed (U) (from token-order.base-token)
    pub base_amount: Amount,        // (from token-order.base-amount)
    pub quote_token: Denom,         // 0xe5cf1 (eU) (from send-call.quote-token)
    pub quote_amount: Amount,       // (from send-call.quote-amount)
    pub remote_base_token: Denom,   // au (from token-order.quote-token)
    pub remote_base_amount: Amount, // (from token-order.quote-amount)
    pub remote_quote_token: Denom,  // eU union1eueueu (from send-call.base-token)
    pub remote_quote_amount: Amount, // (from send-call.base-amount)
    pub delivery_packet_hash: PacketHash,
    pub packet_shape: PacketShape,
}

pub struct Unbond {
    pub sender_zkgm: AddressZkgm,
    pub sender_canonical: AddressCanonical,
    pub sender_display: AddressDisplay,
    pub receiver_zkgm: AddressZkgm,
    pub receiver_canonical: AddressCanonical,
    pub receiver_display: AddressDisplay,
    pub base_token: Denom,     // 0xe5cf1 (eU) (from token-order.base-token)
    pub base_amount: Amount,   // (from token-order.base-amount)
    pub unbond_amount: Amount, // (from token-order.quote-amount)
    pub packet_shape: PacketShape,
}

pub struct Instruction {
    pub instruction_index: InstructionIndex,
    pub instruction_hash: InstructionHash,
    pub instruction_type: InstructionType,
    pub path: InstructionRootPath,
    pub salt: InstructionRootSalt,
    pub instruction_path: InstructionPath,
    pub version: InstructionVersion,
    pub opcode: InstructionOpcode,
    pub operand_sender: Option<OperandSender>,
    pub operand_contract_address: Option<OperandContractAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionIndex(pub u64);

impl From<u64> for InstructionIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<usize> for InstructionIndex {
    type Error = IndexerError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(u64::try_from(value).map_err(|_| {
            IndexerError::CannotMapToHandlerDomain(
                "instruction-index".to_string(),
                value.to_string(),
            )
        })?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionHash(#[serde(with = "bytes_as_hex")] pub Bytes);

impl From<Bytes> for InstructionHash {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for InstructionHash {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "instruction-hash")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionType(pub String);

impl From<String> for InstructionType {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionRootPath(#[serde(with = "bytes_as_hex")] pub Bytes);

impl From<Bytes> for InstructionRootPath {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for InstructionRootPath {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "instruction-root-path")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionRootSalt(#[serde(with = "bytes_as_hex")] pub Bytes);

impl From<Bytes> for InstructionRootSalt {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for InstructionRootSalt {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "instruction-root-salt")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionPath(pub String);

impl InstructionPath {
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_indices(&self) -> Result<Vec<u32>, IndexerError> {
        if self.is_root() {
            return Ok(vec![]);
        }

        self.0
            .split('.')
            .map(|segment| {
                segment.parse().map_err(|_| {
                    IndexerError::CannotMapToHandlerDomain(
                        "instruction-path".to_string(),
                        self.0.clone(),
                    )
                })
            })
            .collect()
    }
}

impl From<&String> for InstructionPath {
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}

impl Display for InstructionPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct InstructionVersion(pub u8);

impl From<u8> for InstructionVersion {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Display for InstructionVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct InstructionOpcode(pub u8);

impl From<u8> for InstructionOpcode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Display for InstructionOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperandSender(pub Bytes);

impl From<Bytes> for OperandSender {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for OperandSender {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "operand-sender")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperandContractAddress(pub Bytes);

impl From<Bytes> for OperandContractAddress {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for OperandContractAddress {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "operand-contract-address")?.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpcType {
    #[serde(rename = "cosmos")]
    Cosmos,
    #[serde(rename = "evm")]
    Evm,
}

impl TryFrom<String> for RpcType {
    type Error = IndexerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.deref() {
            "cosmos" => Ok(RpcType::Cosmos),
            "evm" => Ok(RpcType::Evm),
            unsupported => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "rpc-type".to_string(),
                format!("unsupported {unsupported}"),
            )),
        }
    }
}

impl TryFrom<Option<String>> for RpcType {
    type Error = IndexerError;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        match value {
            Some(rpc_type) => rpc_type.try_into(),
            None => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "rpc-type".to_string(),
                "expecting Some, but got None".to_string(),
            )),
        }
    }
}

impl Display for RpcType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RpcType::Cosmos => "cosmos",
                RpcType::Evm => "evm",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferIndex(pub u32);

impl From<u32> for TransferIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl TryFrom<usize> for TransferIndex {
    type Error = IndexerError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(u32::try_from(value).map_err(|_| {
            IndexerError::CannotMapToHandlerDomain("transfer-index".to_string(), value.to_string())
        })?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressZkgm(#[serde(with = "bytes_as_hex")] pub Bytes, pub RpcType);

impl AddressZkgm {
    pub fn from_string_0x(string_0x: &str, rpc_type: RpcType) -> Result<Self, IndexerError> {
        Ok(AddressZkgm(
            string_0x_to_bytes(string_0x, "address-zkgm")?,
            rpc_type,
        ))
    }

    pub fn bytes(&self) -> Bytes {
        self.0.clone()
    }
}

pub fn bytes_to_value(bytes: &Bytes, context: &str) -> Result<Value, IndexerError> {
    serde_json::from_slice(bytes.as_ref()).map_err(|_| {
        IndexerError::ZkgmExpectingInstructionField(
            format!("{context} is json: {}", hex::encode(bytes)),
            hex::encode(bytes),
        )
    })
}

pub fn string_0x_to_bytes(string_0x: &str, context: &str) -> Result<Bytes, IndexerError> {
    let hex_str = string_0x.strip_prefix("0x").ok_or_else(|| {
        IndexerError::HexDecodeErrorExpecting0x(context.to_string(), string_0x.to_string())
    })?;

    // Pad with leading zero if hex string has odd number of characters
    let hex_str_padded = if hex_str.len() % 2 == 1 {
        format!("0{}", hex_str)
    } else {
        hex_str.to_string()
    };

    let vec = hex::decode(hex_str_padded).map_err(|_| {
        IndexerError::HexDecodeErrorInvalidHex(context.to_string(), string_0x.to_string())
    })?;

    Ok(Bytes::from(vec))
}

pub fn string_base64_to_bytes(string_base64: &str, context: &str) -> Result<Bytes, IndexerError> {
    let decoded = general_purpose::STANDARD
        .decode(string_base64)
        .map_err(|_| {
            IndexerError::Base64DecodeErrorInvalidBase64(
                string_base64.to_string(),
                context.to_string(),
            )
        })?;

    // Convert into Bytes
    Ok(Bytes::from(decoded))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressCanonical(#[serde(with = "bytes_as_hex")] pub Bytes);

impl From<Bytes> for AddressCanonical {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&AddressZkgm> for AddressCanonical {
    type Error = IndexerError;

    fn try_from(value: &AddressZkgm) -> Result<Self, Self::Error> {
        Ok(match value.1 {
            RpcType::Cosmos => bech32::decode(std::str::from_utf8(&value.0).map_err(|_| {
                IndexerError::Bech32DecodeErrorInvalidBech32(
                    "denom not utf8".to_string(),
                    encode(&value.0),
                )
            })?)
            .map(|(_, data)| Bytes::from(data).into())
            .map_err(|_| {
                IndexerError::Bech32DecodeErrorInvalidBech32(
                    "denom not bech".to_string(),
                    encode(&value.0),
                )
            })?,
            RpcType::Evm => value.0.clone().into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressDisplay(pub String);

impl From<String> for AddressDisplay {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl TryFrom<&AddressZkgm> for AddressDisplay {
    type Error = IndexerError;

    fn try_from(value: &AddressZkgm) -> Result<Self, Self::Error> {
        Ok(match value.1 {
            RpcType::Cosmos => {
                match std::str::from_utf8(&value.0) {
                    Ok(utf8_str) => {
                        // If the string contains null bytes, output as hex instead
                        // TODO: this is to be identical to the postgres implementation.
                        // we should fail (and not convert the packet to a transfer).
                        if utf8_str.contains('\0') {
                            format!("0x{}", hex::encode(value.0.clone())).into()
                        } else {
                            utf8_str.to_string().into()
                        }
                    }
                    Err(_) => {
                        // Fallback to hex encoding if from_utf8 fails
                        format!("0x{}", hex::encode(value.0.clone())).into()
                    }
                }
            }
            RpcType::Evm => format!("0x{}", hex::encode(value.0.clone())).into(),
        })
    }
}

impl TryFrom<&String> for Denom {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "denom")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Amount(pub U256);

impl Amount {
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Amount)
    }
}

impl From<U256> for Amount {
    fn from(value: U256) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for Amount {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(U256::from_be_slice(&string_0x_to_bytes(value, "token")?).into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "u8", from = "u8")]
pub enum CreateWrappedTokenKind {
    Protocol,
    ThirdParty,
    Unsupported(u8),
}

impl CreateWrappedTokenKind {
    pub const PROTOCOL: u8 = 0;
    pub const THIRD_PARTY: u8 = 1;
}

impl From<u8> for CreateWrappedTokenKind {
    fn from(value: u8) -> Self {
        match value {
            CreateWrappedTokenKind::PROTOCOL => CreateWrappedTokenKind::Protocol,
            CreateWrappedTokenKind::THIRD_PARTY => CreateWrappedTokenKind::ThirdParty,
            kind => CreateWrappedTokenKind::Unsupported(kind),
        }
    }
}

impl From<CreateWrappedTokenKind> for u8 {
    fn from(kind: CreateWrappedTokenKind) -> u8 {
        match kind {
            CreateWrappedTokenKind::Protocol => CreateWrappedTokenKind::PROTOCOL,
            CreateWrappedTokenKind::ThirdParty => CreateWrappedTokenKind::THIRD_PARTY,
            CreateWrappedTokenKind::Unsupported(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "u8", from = "u8")]
pub enum TokenOrderKind {
    Initialize,
    Escrow,
    Unescrow,
    Solve,
    Unsupported(u8),
}

impl TokenOrderKind {
    pub const INITIALIZE: u8 = 0;
    pub const ESCROW: u8 = 1;
    pub const UNESCROW: u8 = 2;
    pub const SOLVE: u8 = 3;
}

impl From<u8> for TokenOrderKind {
    fn from(value: u8) -> Self {
        match value {
            TokenOrderKind::INITIALIZE => TokenOrderKind::Initialize,
            TokenOrderKind::ESCROW => TokenOrderKind::Escrow,
            TokenOrderKind::UNESCROW => TokenOrderKind::Unescrow,
            TokenOrderKind::SOLVE => TokenOrderKind::Solve,
            kind => TokenOrderKind::Unsupported(kind),
        }
    }
}

impl From<TokenOrderKind> for u8 {
    fn from(kind: TokenOrderKind) -> u8 {
        match kind {
            TokenOrderKind::Initialize => TokenOrderKind::INITIALIZE,
            TokenOrderKind::Escrow => TokenOrderKind::ESCROW,
            TokenOrderKind::Unescrow => TokenOrderKind::UNESCROW,
            TokenOrderKind::Solve => TokenOrderKind::SOLVE,
            TokenOrderKind::Unsupported(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata(#[serde(with = "bytes_as_hex")] pub Bytes);

impl From<Bytes> for Metadata {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for Metadata {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "metadata")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Implementation(#[serde(with = "bytes_as_hex")] pub Bytes);

impl From<Bytes> for Implementation {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for Implementation {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "implementation")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Initializer(#[serde(with = "bytes_as_hex")] pub Bytes);

impl From<Bytes> for Initializer {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for Initializer {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "initializer")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenName(pub String);

impl From<&String> for TokenName {
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenPath(#[serde(with = "bytes_as_hex")] pub Bytes);

impl From<Bytes> for TokenPath {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl TryFrom<&String> for TokenPath {
    type Error = IndexerError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(string_0x_to_bytes(value, "token-path")?.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenSymbol(pub String);

impl From<&String> for TokenSymbol {
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenDecimals(pub u32);

impl From<u32> for TokenDecimals {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fee {
    #[serde(rename = "instruction")]
    Instruction(Denom, Amount, Option<TokenName>),
    #[serde(rename = "quote_delta")]
    QuoteDelta(Denom, Amount, Option<TokenName>),
    #[serde(rename = "quote_delta_negative")]
    QuoteDeltaNegative,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "swap")]
    Swap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapDirection {
    #[serde(rename = "wrapping")]
    Wrapping,
    #[serde(rename = "unwrapping")]
    Unwrapping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketShape {
    #[serde(rename = "batch_v0_transfer_v0_fee")]
    BatchV0TransferV0Fee,
    #[serde(rename = "transfer_v0")]
    TransferV0,
    #[serde(rename = "batch_v0_transfer_v1")]
    BatchV0TransferV1,
    #[serde(rename = "batch_v0_transfer_v1_fee")]
    BatchV0TransferV1Fee,
    #[serde(rename = "transfer_v1")]
    TransferV1,
    #[serde(rename = "batch_v0_transfer_v2")]
    BatchV0TransferV2,
    #[serde(rename = "batch_v0_transfer_v2_fee")]
    BatchV0TransferV2Fee,
    #[serde(rename = "transfer_v2")]
    TransferV2,
    #[serde(rename = "bond_v2")]
    BondV2,
    #[serde(rename = "unbond_v2")]
    UnbondV2,
}

pub mod bytes_as_hex {
    use bytes::Bytes;
    use hex::{decode, encode};

    use super::*;

    pub fn serialize<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = format!("0x{}", encode(bytes));
        serializer.serialize_str(&hex_string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        let s = s
            .strip_prefix("0x")
            .ok_or_else(|| D::Error::custom("missing 0x prefix"))?;
        let vec = decode(s).map_err(D::Error::custom)?;
        Ok(Bytes::from(vec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_0x_to_bytes_odd_nibbles() {
        // Test odd number of nibbles - should pad with leading zero
        let result = string_0x_to_bytes("0x0", "test").unwrap();
        assert_eq!(result, Bytes::from(vec![0x00]));

        let result = string_0x_to_bytes("0xf", "test").unwrap();
        assert_eq!(result, Bytes::from(vec![0x0f]));

        let result = string_0x_to_bytes("0x123", "test").unwrap();
        assert_eq!(result, Bytes::from(vec![0x01, 0x23]));
    }

    #[test]
    fn test_string_0x_to_bytes_even_nibbles() {
        // Test even number of nibbles - should work as before
        let result = string_0x_to_bytes("0x00", "test").unwrap();
        assert_eq!(result, Bytes::from(vec![0x00]));

        let result = string_0x_to_bytes("0xff", "test").unwrap();
        assert_eq!(result, Bytes::from(vec![0xff]));

        let result = string_0x_to_bytes("0x1234", "test").unwrap();
        assert_eq!(result, Bytes::from(vec![0x12, 0x34]));
    }

    #[test]
    fn test_string_0x_to_bytes_error_cases() {
        // Test missing 0x prefix
        let result = string_0x_to_bytes("123", "test");
        assert!(result.is_err());

        // Test invalid hex characters
        let result = string_0x_to_bytes("0xgg", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_cosmos_zkgm_address_to_address_display_with_null_bytes() {
        // Test conversion of a specific hex string to AddressDisplay
        // This hex string ends with "00" which is a null byte that causes database issues
        let hex_string = "0x78696f6e313830306e78686f6731316661686d6473727278617335706d373733686e777077393765356300";
        let bytes = string_0x_to_bytes(hex_string, "test_address").unwrap();
        let zkgm_address = AddressZkgm(bytes, RpcType::Cosmos);

        let result = AddressDisplay::try_from(&zkgm_address);

        // We expect this to succeed and return hex format due to null byte
        let display = result.expect("Conversion should succeed");
        let address_str = display.0.as_str();

        // With null bytes, should be hex-encoded
        assert!(address_str.starts_with("0x"));
        assert_eq!(address_str, hex_string);

        // Verify no null bytes in the final output
        assert!(!address_str.contains('\0'));

        println!("Address with null byte converted to hex: {}", address_str);
    }

    #[test]
    fn test_cosmos_zkgm_address_to_address_display_no_null_bytes() {
        // Test conversion of a hex string without null bytes (should remain as UTF-8 string)
        let hex_string = "0x78696f6e313830306e78686f6731316661686d6473727278617335706d373733686e777077393765356301";
        let bytes = string_0x_to_bytes(hex_string, "test_address").unwrap();
        let zkgm_address = AddressZkgm(bytes, RpcType::Cosmos);

        let result = AddressDisplay::try_from(&zkgm_address);

        // We expect this to succeed and return UTF-8 format
        let display = result.expect("Conversion should succeed");
        let address_str = display.0.as_str();

        // Without null bytes, should remain as UTF-8 string
        assert!(!address_str.starts_with("0x"));
        assert!(!address_str.contains('\0'));
        assert_eq!(
            address_str,
            "xion1800nxhog11fahmdsrrxas5pm773hnwpw97e5c\u{1}"
        );

        println!(
            "Address without null byte remains as UTF-8: {}",
            address_str
        );
    }

    #[test]
    fn test_evm_zkgm_address_to_address_display() {
        // Test conversion of EVM address to AddressDisplay
        // EVM addresses should always be hex-encoded regardless of content
        let hex_string = "0x742d35Cc6634C0532925a3b8D4ee7c9c2b1e732A";
        let bytes = string_0x_to_bytes(hex_string, "test_address").unwrap();
        let zkgm_address = AddressZkgm(bytes, RpcType::Evm);

        let result = AddressDisplay::try_from(&zkgm_address);

        // EVM addresses should always succeed and be hex-encoded
        let display = result.expect("EVM conversion should succeed");
        let address_str = display.0.as_str();

        // EVM addresses should always be hex format
        assert!(address_str.starts_with("0x"));
        assert_eq!(address_str, hex_string.to_lowercase());

        println!("EVM address converted to hex: {}", address_str);
    }

    #[test]
    fn test_cosmos_zkgm_address_to_address_display_invalid_utf8() {
        // Test conversion with invalid UTF-8 bytes (should fallback to hex)
        let invalid_utf8_bytes = vec![0xFF, 0xFE, 0xFD]; // Invalid UTF-8 sequence
        let zkgm_address = AddressZkgm(invalid_utf8_bytes.clone().into(), RpcType::Cosmos);

        let result = AddressDisplay::try_from(&zkgm_address);

        // Should succeed and return hex format due to invalid UTF-8
        let display = result.expect("Conversion should succeed");
        let address_str = display.0.as_str();

        // Should be hex-encoded due to invalid UTF-8
        assert!(address_str.starts_with("0x"));
        assert_eq!(
            address_str,
            format!("0x{}", hex::encode(invalid_utf8_bytes))
        );

        println!("Invalid UTF-8 address converted to hex: {}", address_str);
    }
}
