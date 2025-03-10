use unionlabs::primitives::{Bytes, H160, H256, U256};
#[cfg(feature = "ssz")]
use {
    crate::chain_spec::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES},
    ::ssz::types::{List, Vector},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub gas_limit: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub gas_used: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub timestamp: u64,
    pub extra_data: Bytes,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    pub transactions_root: H256,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ::ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct ExecutionPayloadHeaderSsz<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub logs_bloom: Vector<u8, C::BYTES_PER_LOGS_BLOOM>,
    pub prev_randao: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub block_number: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub gas_limit: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub gas_used: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub timestamp: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub extra_data: List<u8, C::MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    pub transactions_root: H256,
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

    impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> TryFrom<ExecutionPayloadHeader>
        for ExecutionPayloadHeaderSsz<C>
    {
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
            })
        }
    }

    impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> From<ExecutionPayloadHeaderSsz<C>>
        for ExecutionPayloadHeader
    {
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
            }
        }
    }
}
