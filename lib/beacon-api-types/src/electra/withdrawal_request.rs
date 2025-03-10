use unionlabs::primitives::{H160, H384};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
pub struct WithdrawalRequest {
    pub source_address: H160,
    pub validator_pubkey: H384,
    // TODO: Gwei
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub amount: u64,
}
