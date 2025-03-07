use unionlabs::primitives::{H160, H384};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
pub struct ConsolidationRequest {
    pub source_address: H160,
    pub source_pubkey: H384,
    pub target_pubkey: H384,
}
