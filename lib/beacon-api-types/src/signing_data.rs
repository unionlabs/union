use unionlabs::hash::{hash_v2::Hash, H256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SigningData {
    pub object_root: H256,
    pub domain: Hash<32>,
}
