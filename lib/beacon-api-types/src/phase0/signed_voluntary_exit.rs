use unionlabs::primitives::H768;

use crate::phase0::VoluntaryExit;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: H768,
}
