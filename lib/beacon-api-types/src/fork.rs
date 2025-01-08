use crate::Version;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fork {
    pub version: Version,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub epoch: u64,
}
