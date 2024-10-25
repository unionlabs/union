use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconBlockHeader {
    #[serde(with = "serde_utils::string")]
    pub slot: u64,
    #[serde(with = "serde_utils::string")]
    pub proposer_index: u64,
    pub parent_root: H256,
    pub state_root: H256,
    pub body_root: H256,
}
