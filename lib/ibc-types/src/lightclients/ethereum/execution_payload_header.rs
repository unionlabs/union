#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionPayloadHeader {
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub parent_hash: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub fee_recipient: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub state_root: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub receipts_root: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub logs_bloom: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub prev_randao: Vec<u8>,
    pub block_number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub extra_data: Vec<u8>,
    /// TODO(aeryz): U256
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub base_fee_per_gas: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub block_hash: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub transactions_root: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub withdrawals_root: Vec<u8>,
}

impl From<ExecutionPayloadHeader>
    for protos::union::ibc::lightclients::ethereum::v1::ExecutionPayloadHeader
{
    fn from(value: ExecutionPayloadHeader) -> Self {
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
