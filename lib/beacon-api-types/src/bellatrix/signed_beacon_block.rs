use unionlabs::primitives::H768;

use crate::bellatrix::BeaconBlock;
#[cfg(feature = "ssz")]
use crate::{bellatrix::BeaconBlockSsz, chain_spec::ChainSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignedBeaconBlock {
    pub message: BeaconBlock,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct SignedBeaconBlockSsz<C: ChainSpec> {
    pub message: BeaconBlockSsz<C>,
    pub signature: H768,
}
