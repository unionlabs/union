use unionlabs::hash::hash_v2::Hash;

use crate::fork::Fork;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForkParameters {
    pub genesis_fork_version: Hash<4>,
    pub genesis_slot: u64,
    pub altair: Fork,
    pub bellatrix: Fork,
    pub capella: Fork,
    pub deneb: Fork,
}
