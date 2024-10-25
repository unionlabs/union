use unionlabs::hash::H256;

use crate::Version;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenesisData {
    pub genesis_validators_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub genesis_time: u64,
    pub genesis_fork_version: Version,
}
