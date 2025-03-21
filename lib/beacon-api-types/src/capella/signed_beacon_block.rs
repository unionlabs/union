use unionlabs::primitives::H768;
#[cfg(feature = "ssz")]
use {
    crate::{capella::BeaconBlockSsz, chain_spec::ChainSpec},
    ssz::Ssz,
};

use crate::capella::BeaconBlock;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignedBeaconBlock {
    pub message: BeaconBlock,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct SignedBeaconBlockSsz<C: ChainSpec> {
    pub message: BeaconBlockSsz<C>,
    pub signature: H768,
}
