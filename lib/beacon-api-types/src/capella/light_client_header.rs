use unionlabs::primitives::H256;

use crate::{
    capella::ExecutionPayloadHeader,
    consts::{floorlog2, EXECUTION_PAYLOAD_GINDEX},
    phase0::BeaconBlockHeader,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct LightClientHeader {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionPayloadHeader,
    pub execution_branch: [H256; floorlog2(EXECUTION_PAYLOAD_GINDEX)],
}
