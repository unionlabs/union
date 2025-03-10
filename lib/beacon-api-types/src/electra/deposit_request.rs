use unionlabs::primitives::{H256, H384, H768};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
pub struct DepositRequest {
    pub pubkey: H384,
    pub withdrawal_credentials: H256,
    // TODO: Gwei
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub amount: u64,
    pub signature: H768,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub index: u64,
}
