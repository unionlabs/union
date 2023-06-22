#[derive(Debug, Clone, PartialEq)]
pub struct BeaconBlockHeader {
    pub slot: u64,
    pub proposer_index: u64,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub parent_root: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub state_root: Vec<u8>,
    // #[cfg_attr(feature = "std", serde(with = "::serde_utils::base64"))]
    pub body_root: Vec<u8>,
}

impl From<BeaconBlockHeader> for protos::union::ibc::lightclients::ethereum::v1::BeaconBlockHeader {
    fn from(value: BeaconBlockHeader) -> Self {
        Self {
            slot: value.slot,
            proposer_index: value.proposer_index,
            parent_root: value.parent_root,
            state_root: value.state_root,
            body_root: value.body_root,
        }
    }
}
