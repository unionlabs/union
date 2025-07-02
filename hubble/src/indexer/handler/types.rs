use std::{fmt::Display, ops::Deref};

use bytes::Bytes;
use hex::encode;
use ruint::aliases::U256;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::indexer::{
    api::IndexerError,
    event::types::{
        CanonicalChainId, ChannelId, ChannelVersion, ClientId, ClientType, ConnectionId, Denom,
        PortId, UniversalChainId,
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
    pub base_token_name: TokenName,
    pub base_token_path: TokenPath,
    pub base_token_symbol: TokenSymbol,
    pub base_token_decimals: Option<TokenDecimals>, // only None in transfer v0
    pub quote_token: Denom,
    pub quote_amount: Amount,
    pub fee: Fee,
    pub wrap_direction: Option<WrapDirection>,
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
    pub operand_sender: OperandSender,
    pub operand_contract_address: OperandContractAddress,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct InstructionVersion(pub u8);

impl From<u8> for InstructionVersion {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct InstructionOpcode(pub u8);

impl From<u8> for InstructionOpcode {
    fn from(value: u8) -> Self {
        Self(value)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

fn string_0x_to_bytes(string_0x: &str, context: &str) -> Result<Bytes, IndexerError> {
    let hex_str = string_0x.strip_prefix("0x").ok_or_else(|| {
        IndexerError::HexDecodeErrorExpecting0x(context.to_string(), string_0x.to_string())
    })?;
    let vec = hex::decode(hex_str).map_err(|_| {
        IndexerError::HexDecodeErrorInvalidHex(context.to_string(), string_0x.to_string())
    })?;

    Ok(Bytes::from(vec))
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
                IndexerError::Bech32DecodeErrorInvalidBech32("denom".to_string(), encode(&value.0))
            })?)
            .map(|(_, data)| Bytes::from(data).into())
            .map_err(|_| {
                IndexerError::Bech32DecodeErrorInvalidBech32("denom".to_string(), encode(&value.0))
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
            RpcType::Cosmos => std::str::from_utf8(&value.0)
                .map_err(|_| {
                    IndexerError::Bech32DecodeErrorInvalidBech32(
                        "denom".to_string(),
                        encode(&value.0),
                    )
                })?
                .to_string()
                .into(),
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
    Instruction(Denom, Amount, TokenName),
    #[serde(rename = "quote_delta")]
    QuoteDelta(Denom, Amount, TokenName),
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
