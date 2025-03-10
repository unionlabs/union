use unionlabs::primitives::H768;

use crate::phase0::BeaconBlockHeader;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignedBeaconBlockHeader {
    pub message: BeaconBlockHeader,
    pub signature: H768,
}
