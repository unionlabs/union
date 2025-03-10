use beacon_api_types::{capella, deneb, phase0};
use unionlabs_primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct LightClientHeader {
    /// The beacon block header as used in the latest fork.
    pub beacon: phase0::BeaconBlockHeader,
    /// The execution payload header as used in the latest fork.
    pub execution: deneb::ExecutionPayloadHeader,
    pub execution_branch: Vec<H256>,
}

impl From<capella::LightClientHeader> for LightClientHeader {
    fn from(value: capella::LightClientHeader) -> Self {
        Self {
            beacon: value.beacon,
            execution: deneb::ExecutionPayloadHeader {
                parent_hash: value.execution.parent_hash,
                fee_recipient: value.execution.fee_recipient,
                state_root: value.execution.state_root,
                receipts_root: value.execution.receipts_root,
                logs_bloom: value.execution.logs_bloom,
                prev_randao: value.execution.prev_randao,
                block_number: value.execution.block_number,
                gas_limit: value.execution.gas_limit,
                gas_used: value.execution.gas_used,
                timestamp: value.execution.timestamp,
                extra_data: value.execution.extra_data,
                base_fee_per_gas: value.execution.base_fee_per_gas,
                block_hash: value.execution.block_hash,
                transactions_root: value.execution.transactions_root,
                withdrawals_root: value.execution.withdrawals_root,
                blob_gas_used: 0,
                excess_blob_gas: 0,
            },
            execution_branch: value.execution_branch.to_vec(),
        }
    }
}

impl From<deneb::LightClientHeader> for LightClientHeader {
    fn from(value: deneb::LightClientHeader) -> Self {
        Self {
            beacon: value.beacon,
            execution: value.execution,
            execution_branch: value.execution_branch.to_vec(),
        }
    }
}
