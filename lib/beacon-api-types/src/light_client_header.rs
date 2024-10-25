use unionlabs::hash::H256;

use crate::{
    beacon_block_header::BeaconBlockHeader,
    consts::{floorlog2, EXECUTION_PAYLOAD_INDEX},
    execution_payload_header::ExecutionPayloadHeader,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LightClientHeader {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionPayloadHeader,
    pub execution_branch: [H256; floorlog2(EXECUTION_PAYLOAD_INDEX)],
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct LightClientHeaderSsz<C: crate::BYTES_PER_LOGS_BLOOM + crate::MAX_EXTRA_DATA_BYTES> {
    pub beacon: BeaconBlockHeader,
    pub execution: crate::ExecutionPayloadHeaderSsz<C>,
    pub execution_branch: [H256; floorlog2(EXECUTION_PAYLOAD_INDEX)],
}
