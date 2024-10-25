use unionlabs::bls::BlsSignature;

use crate::BlsToExecutionChange;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
pub struct SignedBlsToExecutionChange {
    message: BlsToExecutionChange,
    signature: BlsSignature,
}
