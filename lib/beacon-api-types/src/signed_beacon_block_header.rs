use unionlabs::primitives::H768;

use crate::BeaconBlockHeader;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignedBeaconBlockHeader {
    pub message: BeaconBlockHeader,
    pub signature: H768,
}
