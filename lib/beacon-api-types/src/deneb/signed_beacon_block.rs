use unionlabs::primitives::H768;

use crate::deneb::BeaconBlock;
#[cfg(feature = "ssz")]
use crate::{chain_spec::ChainSpec, deneb::BeaconBlockSsz};

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
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct SignedBeaconBlockSsz<C: ChainSpec> {
    pub message: BeaconBlockSsz<C>,
    pub signature: H768,
}
