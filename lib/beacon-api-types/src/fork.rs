use unionlabs::hash::hash_v2::Hash;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fork {
    pub version: Hash<4>,
    pub epoch: u64,
}
