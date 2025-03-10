use unionlabs::primitives::H768;

use crate::capella::BlsToExecutionChange;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
pub struct SignedBlsToExecutionChange {
    message: BlsToExecutionChange,
    signature: H768,
}
