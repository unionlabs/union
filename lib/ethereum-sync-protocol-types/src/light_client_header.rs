use beacon_api_types::{deneb, phase0};
use unionlabs_primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct LightClientHeader {
    /// The beacon block header as used in the latest fork.
    pub beacon: phase0::BeaconBlockHeader,
    /// The execution payload header as used in the latest fork.
    pub execution: deneb::ExecutionPayloadHeader,
    pub execution_branch: Vec<H256>,
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
