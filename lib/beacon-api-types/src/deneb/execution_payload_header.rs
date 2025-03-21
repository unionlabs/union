use unionlabs::primitives::{Bytes, H160, H256, U256};
#[cfg(feature = "ssz")]
use {
    crate::chain_spec::ChainSpec,
    ::ssz::types::{List, Vector},
};

use crate::custom_types::Gas;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ExecutionPayloadHeader {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    pub logs_bloom: Bytes,
    pub prev_randao: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub block_number: u64,
    pub gas_limit: Gas,
    pub gas_used: Gas,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub timestamp: u64,
    pub extra_data: Bytes,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    pub transactions_root: H256,
    pub withdrawals_root: H256,
    pub blob_gas_used: Gas,
    pub excess_blob_gas: Gas,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ::ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct ExecutionPayloadHeaderSsz<C: ChainSpec> {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub logs_bloom: Vector<u8, C::BYTES_PER_LOGS_BLOOM>,
    pub prev_randao: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub block_number: u64,
    pub gas_limit: Gas,
    pub gas_used: Gas,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub timestamp: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub extra_data: List<u8, C::MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    pub transactions_root: H256,
    pub withdrawals_root: H256,
    pub blob_gas_used: Gas,
    pub excess_blob_gas: Gas,
}

#[cfg(feature = "ssz")]
pub mod ssz {
    use typenum::Unsigned;
    use unionlabs::errors::{ExpectedLength, InvalidLength};

    use super::*;

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid logs bloom")]
        LogsBloom(#[source] InvalidLength),

        #[error("invalid extra data")]
        ExtraData(#[source] InvalidLength),
    }

    impl<C: ChainSpec> TryFrom<ExecutionPayloadHeader> for ExecutionPayloadHeaderSsz<C> {
        type Error = Error;

        fn try_from(value: ExecutionPayloadHeader) -> Result<Self, Self::Error> {
            Ok(Self {
                parent_hash: value.parent_hash,
                fee_recipient: value.fee_recipient,
                state_root: value.state_root,
                receipts_root: value.receipts_root,
                logs_bloom: value
                    .logs_bloom
                    .into_vec()
                    .try_into()
                    .map_err(|v: Vec<_>| {
                        Error::LogsBloom(InvalidLength {
                            expected: ExpectedLength::Exact(C::BYTES_PER_LOGS_BLOOM::USIZE),
                            found: v.len(),
                        })
                    })?,
                prev_randao: value.prev_randao,
                block_number: value.block_number,
                gas_limit: value.gas_limit,
                gas_used: value.gas_used,
                timestamp: value.timestamp,
                extra_data: value
                    .extra_data
                    .into_vec()
                    .try_into()
                    .map_err(|l: Vec<_>| {
                        Error::ExtraData(InvalidLength {
                            expected: ExpectedLength::Between(0, C::MAX_EXTRA_DATA_BYTES::USIZE),
                            found: l.len(),
                        })
                    })?,
                base_fee_per_gas: value.base_fee_per_gas,
                block_hash: value.block_hash,
                transactions_root: value.transactions_root,
                withdrawals_root: value.withdrawals_root,
                blob_gas_used: value.blob_gas_used,
                excess_blob_gas: value.excess_blob_gas,
            })
        }
    }

    impl<C: ChainSpec> From<ExecutionPayloadHeaderSsz<C>> for ExecutionPayloadHeader {
        fn from(value: ExecutionPayloadHeaderSsz<C>) -> Self {
            Self {
                parent_hash: value.parent_hash,
                fee_recipient: value.fee_recipient,
                state_root: value.state_root,
                receipts_root: value.receipts_root,
                logs_bloom: value.logs_bloom.into_iter().collect(),
                prev_randao: value.prev_randao,
                block_number: value.block_number,
                gas_limit: value.gas_limit,
                gas_used: value.gas_used,
                timestamp: value.timestamp,
                extra_data: value.extra_data.into_iter().collect(),
                base_fee_per_gas: value.base_fee_per_gas,
                block_hash: value.block_hash,
                transactions_root: value.transactions_root,
                withdrawals_root: value.withdrawals_root,
                blob_gas_used: value.blob_gas_used,
                excess_blob_gas: value.excess_blob_gas,
            }
        }
    }
}
