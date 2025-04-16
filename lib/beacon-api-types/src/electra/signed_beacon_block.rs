use unionlabs::primitives::H768;
#[cfg(feature = "ssz")]
use {
    crate::{chain_spec::ChainSpec, electra::BeaconBlockSsz},
    ssz::Ssz,
};

use crate::electra::BeaconBlock;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct SignedBeaconBlock {
    pub message: BeaconBlock,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct SignedBeaconBlockSsz<C: ChainSpec> {
    pub message: BeaconBlockSsz<C>,
    pub signature: H768,
}
