use unionlabs::primitives::H256;

use crate::custom_types::{Slot, ValidatorIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct BeaconBlockHeader {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
}
