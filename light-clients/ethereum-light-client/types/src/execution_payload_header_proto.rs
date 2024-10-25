use beacon_api_types::execution_payload_header::ExecutionPayloadHeader;
use unionlabs::{errors::InvalidLength, uint::U256};

pub fn into_proto(
    value: ExecutionPayloadHeader,
) -> protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader {
    protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader {
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

pub fn try_from_proto(
    value: protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader,
) -> Result<ExecutionPayloadHeader, Error> {
    Ok(ExecutionPayloadHeader {
        parent_hash: value.parent_hash.try_into().map_err(Error::ParentHash)?,
        fee_recipient: value
            .fee_recipient
            .try_into()
            .map_err(Error::FeeRecipient)?,
        state_root: value.state_root.try_into().map_err(Error::StateRoot)?,
        receipts_root: value
            .receipts_root
            .try_into()
            .map_err(Error::ReceiptsRoot)?,
        logs_bloom: value.logs_bloom,
        prev_randao: value.prev_randao.try_into().map_err(Error::PrevRandao)?,
        block_number: value.block_number,
        gas_limit: value.gas_limit,
        gas_used: value.gas_used,
        timestamp: value.timestamp,
        extra_data: value.extra_data,
        base_fee_per_gas: U256::try_from_be_bytes(&value.base_fee_per_gas)
            .map_err(Error::BaseFeePerGas)?,
        block_hash: value.block_hash.try_into().map_err(Error::BlockHash)?,
        transactions_root: value
            .transactions_root
            .try_into()
            .map_err(Error::TransactionsRoot)?,
        withdrawals_root: value
            .withdrawals_root
            .try_into()
            .map_err(Error::WithdrawalsRoot)?,
        blob_gas_used: value.blob_gas_used,
        excess_blob_gas: value.excess_blob_gas,
    })
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum Error {
    #[error("invalid parent_hash")]
    ParentHash(#[source] InvalidLength),
    #[error("invalid fee_recipient")]
    FeeRecipient(#[source] InvalidLength),
    #[error("invalid state_root")]
    StateRoot(#[source] InvalidLength),
    #[error("invalid receipts_root")]
    ReceiptsRoot(#[source] InvalidLength),
    #[error("invalid prev_randao")]
    PrevRandao(#[source] InvalidLength),
    #[error("invalid base_fee_per_gas")]
    BaseFeePerGas(#[source] InvalidLength),
    #[error("invalid block_hash")]
    BlockHash(#[source] InvalidLength),
    #[error("invalid transactions_root")]
    TransactionsRoot(#[source] InvalidLength),
    #[error("invalid withdrawals_root")]
    WithdrawalsRoot(#[source] InvalidLength),
}
