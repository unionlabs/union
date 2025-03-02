use unionlabs::primitives::H768;

use crate::VoluntaryExit;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: H768,
}
