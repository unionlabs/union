use primitive_types::U256;
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_types::{fixed_vector, variable_list, FixedVector, VariableList};
use tree_hash::TreeHash;

use crate::{
    errors::InvalidLength,
    ethereum::{Address, H256},
    ethereum_consts_traits::{BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES},
    Proto, TypeUrl,
};

#[derive(Clone, Debug, PartialEq, Encode, Decode, TreeHash, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ExecutionPayloadHeader<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> {
    pub parent_hash: H256,
    pub fee_recipient: Address,
    pub state_root: H256,
    pub receipts_root: H256,
    #[serde(with = "::serde_utils::hex_string")]
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
    pub extra_data: VariableList<u8, C::MAX_EXTRA_DATA_BYTES>,
    #[serde(with = "::serde_utils::u256_from_dec_str")]
    pub base_fee_per_gas: U256,
    pub block_hash: H256,
    #[serde(default)]
    pub transactions_root: H256,
    #[serde(default)]
    pub withdrawals_root: H256,
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
            // base_fee_per_gas: <[u8; 32]>::from(value.base_fee_per_gas).into(),
            base_fee_per_gas: {
                let mut slice = [0_u8; 32];
                value.base_fee_per_gas.to_little_endian(&mut slice);
                slice.into()
            },
            block_hash: value.block_hash.into(),
            transactions_root: value.transactions_root.into(),
            withdrawals_root: value.withdrawals_root.into(),
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
    // contains the invalid bytes
    BaseFeePerGas(Vec<u8>),
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
            base_fee_per_gas: U256::from_little_endian(
                &<[u8; 32]>::try_from(value.base_fee_per_gas)
                    .map_err(TryFromExecutionPayloadHeaderError::BaseFeePerGas)?,
            ),
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
        })
    }
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.ExecutionPayloadHeader";
}

impl<C: BYTES_PER_LOGS_BLOOM + MAX_EXTRA_DATA_BYTES> Proto for ExecutionPayloadHeader<C> {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader;
}
