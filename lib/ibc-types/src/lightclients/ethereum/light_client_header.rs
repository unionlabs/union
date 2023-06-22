use crate::lightclients::ethereum::{
    beacon_block_header::BeaconBlockHeader, execution_payload_header::ExecutionPayloadHeader,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LightClientHeader {
    pub beacon: BeaconBlockHeader,
    pub execution: ExecutionPayloadHeader,
    // #[serde(with = "::serde_utils::inner_base64")]
    pub execution_branch: Vec<Vec<u8>>,
}

impl From<LightClientHeader> for protos::union::ibc::lightclients::ethereum::v1::LightClientHeader {
    fn from(value: LightClientHeader) -> Self {
        Self {
            beacon: Some(value.beacon.into()),
            execution: Some(value.execution.into()),
            execution_branch: value.execution_branch,
        }
    }
}
