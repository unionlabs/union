use unionlabs::primitives::{Bytes, H160, H256, U256};
#[cfg(feature = "ssz")]
use {
    crate::chain_spec::ChainSpec,
    ssz::types::{List, Vector},
};

use crate::{capella::Withdrawal, custom_types::Gas};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct ExecutionPayload {
    /// Execution block header fields
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    pub logs_bloom: Bytes,
    /// 'difficulty' in the yellow paper
    pub prev_randao: H256,
    /// 'number' in the yellow paper
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub block_number: u64,
    pub gas_limit: Gas,
    pub gas_used: Gas,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub timestamp: u64,
    pub extra_data: Bytes,
    pub base_fee_per_gas: U256,
    /// Extra payload fields
    /// Hash of execution block
    pub block_hash: H256,
    pub transactions: Vec<Bytes>,
    pub withdrawals: Vec<Withdrawal>,
    pub blob_gas_used: Gas,
    pub excess_blob_gas: Gas,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct ExecutionPayloadSsz<C: ChainSpec> {
    /// Execution block header fields
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub logs_bloom: Vector<u8, C::BYTES_PER_LOGS_BLOOM>,
    /// 'difficulty' in the yellow paper
    pub prev_randao: H256,
    /// 'number' in the yellow paper
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub block_number: u64,
    pub gas_limit: Gas,
    pub gas_used: Gas,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub timestamp: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub extra_data: List<u8, C::MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    /// Extra payload fields
    /// Hash of execution block
    pub block_hash: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string_list"))]
    pub transactions: List<List<u8, C::MAX_BYTES_PER_TRANSACTION>, C::MAX_TRANSACTIONS_PER_PAYLOAD>,
    pub withdrawals: List<Withdrawal, C::MAX_WITHDRAWALS_PER_PAYLOAD>,
    pub blob_gas_used: Gas,
    pub excess_blob_gas: Gas,
}
