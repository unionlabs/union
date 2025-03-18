use unionlabs::primitives::H256;

use crate::custom_types::Epoch;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct Checkpoint {
    pub epoch: Epoch,
    pub root: H256,
}
