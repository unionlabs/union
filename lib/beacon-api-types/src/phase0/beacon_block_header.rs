use unionlabs::primitives::H256;

use crate::slot::Slot;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct BeaconBlockHeader {
    pub slot: Slot,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
}
