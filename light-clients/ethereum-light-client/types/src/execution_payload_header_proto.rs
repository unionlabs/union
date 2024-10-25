pub fn into_proto(
    value: ExecutionPayloadHeader<C>,
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
        logs_bloom: value.logs_bloom.try_into().map_err(|vec: Vec<_>| {
            Error::LogsBloom(InvalidLength {
                expected: ExpectedLength::Exact(C::BYTES_PER_LOGS_BLOOM::USIZE),
                found: vec.len(),
            })
        })?,
        prev_randao: value.prev_randao.try_into().map_err(Error::PrevRandao)?,
        block_number: value.block_number,
        gas_limit: value.gas_limit,
        gas_used: value.gas_used,
        timestamp: value.timestamp,
        extra_data: value.extra_data.try_into().map_err(|vec: Vec<_>| {
            Error::ExtraData(InvalidLength {
                expected: ExpectedLength::Exact(C::MAX_EXTRA_DATA_BYTES::USIZE),
                found: vec.len(),
            })
        })?,
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
