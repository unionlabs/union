use ssz::types::{List, Vector};
use unionlabs::{
    hash::{H160, H256},
    uint::U256,
};

#[cfg(feature = "ssz")]
use crate::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecutionPayloadHeader {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[serde(with = "::serde_utils::hex_string")]
    // #[debug("{}", serde_utils::to_hex(&logs_bloom))]
    pub logs_bloom: Vec<u8>,
    pub prev_randao: H256,
    #[serde(with = "::serde_utils::string")]
    pub block_number: u64,
    #[serde(with = "::serde_utils::string")]
    pub gas_limit: u64,
    #[serde(with = "::serde_utils::string")]
    pub gas_used: u64,
    #[serde(with = "::serde_utils::string")]
    pub timestamp: u64,
    #[serde(with = "::serde_utils::hex_string")]
    // #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub extra_data: Vec<u8>,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    #[serde(default)]
    pub transactions_root: H256,
    #[serde(default)]
    pub withdrawals_root: H256,
    // blob_gas_used: uint64  # [New in Deneb:EIP4844]
    #[serde(default, with = "::serde_utils::string")]
    pub blob_gas_used: u64,
    // excess_blob_gas: uint64  # [New in Deneb:EIP4844]
    #[serde(default, with = "::serde_utils::string")]
    pub excess_blob_gas: u64,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExecutionPayloadHeaderSsz<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[serde(with = "::serde_utils::hex_string")]
    // #[debug("{}", serde_utils::to_hex(&logs_bloom))]
    pub logs_bloom: Vector<u8, C::BYTES_PER_LOGS_BLOOM>,
    pub prev_randao: H256,
    #[serde(with = "::serde_utils::string")]
    pub block_number: u64,
    #[serde(with = "::serde_utils::string")]
    pub gas_limit: u64,
    #[serde(with = "::serde_utils::string")]
    pub gas_used: u64,
    #[serde(with = "::serde_utils::string")]
    pub timestamp: u64,
    #[serde(with = "::serde_utils::hex_string")]
    // #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub extra_data: List<u8, C::MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    #[serde(default)]
    pub transactions_root: H256,
    #[serde(default)]
    pub withdrawals_root: H256,
    // blob_gas_used: uint64  # [New in Deneb:EIP4844]
    #[serde(default, with = "::serde_utils::string")]
    pub blob_gas_used: u64,
    // excess_blob_gas: uint64  # [New in Deneb:EIP4844]
    #[serde(default, with = "::serde_utils::string")]
    pub excess_blob_gas: u64,
}
