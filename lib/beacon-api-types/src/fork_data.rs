use unionlabs::hash::{hash_v2::Hash, H256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForkData {
    pub current_version: Hash<4>,
    pub genesis_validators_root: H256,
}
