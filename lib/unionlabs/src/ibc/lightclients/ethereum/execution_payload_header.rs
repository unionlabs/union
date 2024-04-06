use macros::model;
use ssz::types::{fixed_vector, variable_list, FixedVector, VariableList};
use tree_hash::TreeHash;

use crate::{
    errors::InvalidLength,
    ethereum::config::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES},
    hash::{H160, H256},
    uint::U256,
};

#[derive(ssz::Encode, ssz::Decode, TreeHash)]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::AccountUpdate),
    into,
    from
))]
pub struct CapellaExecutionPayloadHeader<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug("{}", serde_utils::to_hex(&logs_bloom))]
    pub logs_bloom: FixedVector<u8, C::BYTES_PER_LOGS_BLOOM>,
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
    #[debug("{}", serde_utils::to_hex(&extra_data))]
    pub extra_data: VariableList<u8, C::MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    #[serde(default)]
    pub transactions_root: H256,
    #[serde(default)]
    pub withdrawals_root: H256,
}

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

#[model]
#[derive(ssz::Encode, ssz::Decode, TreeHash)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ExecutionPayloadHeader<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub parent_hash: H256,
    pub fee_recipient: H160,
    pub state_root: H256,
    pub receipts_root: H256,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug("{}", serde_utils::to_hex(&logs_bloom))]
    pub logs_bloom: FixedVector<u8, C::BYTES_PER_LOGS_BLOOM>,
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
    #[debug("{}", serde_utils::to_hex(&extra_data))]
    pub extra_data: VariableList<u8, C::MAX_EXTRA_DATA_BYTES>,
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
            base_fee_per_gas: value.base_fee_per_gas.to_big_endian().into(),
            block_hash: value.block_hash.into(),
            transactions_root: value.transactions_root.into(),
            withdrawals_root: value.withdrawals_root.into(),
            blob_gas_used: value.blob_gas_used,
            excess_blob_gas: value.excess_blob_gas,
        }
    }
}

#[derive(Debug)]
pub enum TryFromExecutionPayloadHeaderError {
    ParentHash(InvalidLength),
    FeeRecipient(InvalidLength),
    StateRoot(InvalidLength),
    ReceiptsRoot(InvalidLength),
    LogsBloom(fixed_vector::TryFromVecError),
    PrevRandao(InvalidLength),
    ExtraData(variable_list::TryFromVecError),
    BaseFeePerGas(InvalidLength),
    BlockHash(InvalidLength),
    TransactionsRoot(InvalidLength),
    WithdrawalsRoot(InvalidLength),
}

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
            logs_bloom: value
                .logs_bloom
                .try_into()
                .map_err(TryFromExecutionPayloadHeaderError::LogsBloom)?,
            prev_randao: value
                .prev_randao
                .try_into()
                .map_err(TryFromExecutionPayloadHeaderError::PrevRandao)?,
            block_number: value.block_number,
            gas_limit: value.gas_limit,
            gas_used: value.gas_used,
            timestamp: value.timestamp,
            extra_data: value
                .extra_data
                .try_into()
                .map_err(TryFromExecutionPayloadHeaderError::ExtraData)?,
            base_fee_per_gas: U256::try_from_big_endian(&value.base_fee_per_gas)
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
