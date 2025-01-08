use crate::{fork::Fork, Version};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForkParameters {
    pub genesis_fork_version: Version,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub genesis_slot: u64,
    pub altair: Fork,
    pub bellatrix: Fork,
    pub capella: Fork,
    pub deneb: Fork,
}
