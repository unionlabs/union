use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlockHeader {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub slot: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
}
