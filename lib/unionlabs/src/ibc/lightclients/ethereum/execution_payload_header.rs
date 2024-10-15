use macros::model;
#[cfg(feature = "ssz")]
use {
    crate::ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES},
    ssz::{
        types::{List, Vector},
        Ssz,
    },
};

use crate::{
    hash::{H160, H256},
    uint::U256,
};

#[cfg(feature = "ssz")]
#[model]
#[derive(Ssz)]
pub struct CapellaExecutionPayloadHeader<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug("{}", serde_utils::to_hex(&logs_bloom))]
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
    #[debug("{}", serde_utils::to_hex(&extra_data))]
    pub extra_data: List<u8, C::MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    #[cfg_attr(feature = "serde", serde(default))]
    pub transactions_root: H256,
    #[cfg_attr(feature = "serde", serde(default))]
    pub withdrawals_root: H256,
}

#[cfg(feature = "ssz")]
impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> From<ExecutionPayloadHeader<C>>
    for CapellaExecutionPayloadHeader<C>
{
    fn from(value: ExecutionPayloadHeader<C>) -> Self {
        Self {
            parent_hash: value.parent_hash,
            fee_recipient: value.fee_recipient,
            state_root: value.state_root,
            receipts_root: value.receipts_root,
            logs_bloom: value.logs_bloom,
            prev_randao: value.prev_randao,
            block_number: value.block_number,
            gas_limit: value.gas_limit,
            gas_used: value.gas_used,
            timestamp: value.timestamp,
            extra_data: value.extra_data,
            base_fee_per_gas: value.base_fee_per_gas,
            block_hash: value.block_hash,
            transactions_root: value.transactions_root,
            withdrawals_root: value.withdrawals_root,
        }
    }
}

#[cfg(feature = "ssz")]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader),
    into,
    from
))]
#[derive(::ssz::Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct ExecutionPayloadHeader<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug("{}", serde_utils::to_hex(&logs_bloom))]
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
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub extra_data: List<u8, C::MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    #[cfg_attr(feature = "serde", serde(default))]
    pub transactions_root: H256,
    #[cfg_attr(feature = "serde", serde(default))]
    pub withdrawals_root: H256,
    // blob_gas_used: uint64  # [New in Deneb:EIP4844]
    #[cfg_attr(feature = "serde", serde(default, with = "::serde_utils::string"))]
    pub blob_gas_used: u64,
    // excess_blob_gas: uint64  # [New in Deneb:EIP4844]
    #[cfg_attr(feature = "serde", serde(default, with = "::serde_utils::string"))]
    pub excess_blob_gas: u64,
}

// TODO: Ssz encoding doesn't need to take ownership, impl for &T as well as T
// TODO: Impl this via #[model]
#[cfg(feature = "ssz")]
impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> crate::encoding::Decode<crate::encoding::Ssz>
    for ExecutionPayloadHeader<C>
{
    type Error = ssz::decode::DecodeError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        <Self as ssz::Ssz>::from_ssz_bytes(bytes)
    }
}

// TODO: Impl this via #[model]
#[cfg(feature = "ssz")]
impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> crate::encoding::Encode<crate::encoding::Ssz>
    for ExecutionPayloadHeader<C>
{
    fn encode(self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    #[cfg(feature = "ssz")]
    use {
        crate::{
            errors::{ExpectedLength, InvalidLength},
            ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES},
            ibc::lightclients::ethereum::execution_payload_header::ExecutionPayloadHeader,
            uint::U256,
        },
        typenum::Unsigned,
    };

    use crate::ibc::lightclients::ethereum::execution_payload_header::UnboundedExecutionPayloadHeader;

    #[cfg(feature = "ssz")]
    impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> From<ExecutionPayloadHeader<C>>
        for protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader
    {
        fn from(value: ExecutionPayloadHeader<C>) -> Self {
            Self {
                parent_hash: value.parent_hash.into(),
                fee_recipient: value.fee_recipient.into(),
                state_root: value.state_root.into(),
                receipts_root: value.receipts_root.into(),
                logs_bloom: value.logs_bloom.into(),
                prev_randao: value.prev_randao.into(),
                block_number: value.block_number,
                gas_limit: value.gas_limit,
                gas_used: value.gas_used,
                timestamp: value.timestamp,
                extra_data: value.extra_data.into(),
                base_fee_per_gas: value.base_fee_per_gas.to_be_bytes().into(),
                block_hash: value.block_hash.into(),
                transactions_root: value.transactions_root.into(),
                withdrawals_root: value.withdrawals_root.into(),
                blob_gas_used: value.blob_gas_used,
                excess_blob_gas: value.excess_blob_gas,
            }
        }
    }

    #[cfg(feature = "ssz")]
    impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES>
        TryFrom<protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader>
        for ExecutionPayloadHeader<C>
    {
        type Error = TryFromExecutionPayloadHeaderError;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                parent_hash: value
                    .parent_hash
                    .try_into()
                    .map_err(TryFromExecutionPayloadHeaderError::ParentHash)?,
                fee_recipient: value
                    .fee_recipient
                    .try_into()
                    .map_err(TryFromExecutionPayloadHeaderError::FeeRecipient)?,
                state_root: value
                    .state_root
                    .try_into()
                    .map_err(TryFromExecutionPayloadHeaderError::StateRoot)?,
                receipts_root: value
                    .receipts_root
                    .try_into()
                    .map_err(TryFromExecutionPayloadHeaderError::ReceiptsRoot)?,
                logs_bloom: value.logs_bloom.try_into().map_err(|vec: Vec<_>| {
                    TryFromExecutionPayloadHeaderError::LogsBloom(InvalidLength {
                        expected: ExpectedLength::Exact(C::BYTES_PER_LOGS_BLOOM::USIZE),
                        found: vec.len(),
                    })
                })?,
                prev_randao: value
                    .prev_randao
                    .try_into()
                    .map_err(TryFromExecutionPayloadHeaderError::PrevRandao)?,
                block_number: value.block_number,
                gas_limit: value.gas_limit,
                gas_used: value.gas_used,
                timestamp: value.timestamp,
                extra_data: value.extra_data.try_into().map_err(|vec: Vec<_>| {
                    TryFromExecutionPayloadHeaderError::ExtraData(InvalidLength {
                        expected: ExpectedLength::Exact(C::MAX_EXTRA_DATA_BYTES::USIZE),
                        found: vec.len(),
                    })
                })?,
                base_fee_per_gas: U256::try_from_be_bytes(&value.base_fee_per_gas)
                    .map_err(TryFromExecutionPayloadHeaderError::BaseFeePerGas)?,
                block_hash: value
                    .block_hash
                    .try_into()
                    .map_err(TryFromExecutionPayloadHeaderError::BlockHash)?,
                transactions_root: value
                    .transactions_root
                    .try_into()
                    .map_err(TryFromExecutionPayloadHeaderError::TransactionsRoot)?,
                withdrawals_root: value
                    .withdrawals_root
                    .try_into()
                    .map_err(TryFromExecutionPayloadHeaderError::WithdrawalsRoot)?,
                blob_gas_used: value.blob_gas_used,
                excess_blob_gas: value.excess_blob_gas,
            })
        }
    }

    #[cfg(feature = "ssz")]
    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromExecutionPayloadHeaderError {
        #[error("invalid parent hash")]
        ParentHash(#[source] InvalidLength),
        #[error("invalid fee recipient")]
        FeeRecipient(#[source] InvalidLength),
        #[error("invalid state root")]
        StateRoot(#[source] InvalidLength),
        #[error("invalid receipts root")]
        ReceiptsRoot(#[source] InvalidLength),
        #[error("invalid logs bloom")]
        LogsBloom(#[source] InvalidLength),
        #[error("invalid prev randao")]
        PrevRandao(#[source] InvalidLength),
        #[error("invalid extra data")]
        ExtraData(#[source] InvalidLength),
        #[error("invalid base fee per gas")]
        BaseFeePerGas(#[source] InvalidLength),
        #[error("invalid block hash")]
        BlockHash(#[source] InvalidLength),
        #[error("invalid transactions root")]
        TransactionsRoot(#[source] InvalidLength),
        #[error("invalid withdrawals root")]
        WithdrawalsRoot(#[source] InvalidLength),
    }

    impl From<UnboundedExecutionPayloadHeader>
        for protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader
    {
        fn from(value: UnboundedExecutionPayloadHeader) -> Self {
            Self {
                parent_hash: value.parent_hash.into(),
                fee_recipient: value.fee_recipient.into(),
                state_root: value.state_root.into(),
                receipts_root: value.receipts_root.into(),
                logs_bloom: value.logs_bloom,
                prev_randao: value.prev_randao.into(),
                block_number: value.block_number,
                gas_limit: value.gas_limit,
                gas_used: value.gas_used,
                timestamp: value.timestamp,
                extra_data: value.extra_data,
                base_fee_per_gas: value.base_fee_per_gas.to_be_bytes().into(),
                block_hash: value.block_hash.into(),
                transactions_root: value.transactions_root.into(),
                withdrawals_root: value.withdrawals_root.into(),
                blob_gas_used: value.blob_gas_used,
                excess_blob_gas: value.excess_blob_gas,
            }
        }
    }
}

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader),
    from
))]
pub struct UnboundedExecutionPayloadHeader {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    #[debug("{}", serde_utils::to_hex(&logs_bloom))]
    pub logs_bloom: Vec<u8>,
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
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub extra_data: Vec<u8>,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    #[cfg_attr(feature = "serde", serde(default))]
    pub transactions_root: H256,
    #[cfg_attr(feature = "serde", serde(default))]
    pub withdrawals_root: H256,
    // blob_gas_used: uint64  # [New in Deneb:EIP4844]
    #[cfg_attr(feature = "serde", serde(default, with = "::serde_utils::string"))]
    pub blob_gas_used: u64,
    // excess_blob_gas: uint64  # [New in Deneb:EIP4844]
    #[cfg_attr(feature = "serde", serde(default, with = "::serde_utils::string"))]
    pub excess_blob_gas: u64,
}
