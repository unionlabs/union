use unionlabs::primitives::H256;

use crate::custom_types::Domain;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct SigningData {
    pub object_root: H256,
    pub domain: Domain,
}
