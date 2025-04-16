use crate::custom_types::{Epoch, ValidatorIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct VoluntaryExit {
    /// Earliest epoch when voluntary exit can be processed
    pub epoch: Epoch,
    pub validator_index: ValidatorIndex,
}
