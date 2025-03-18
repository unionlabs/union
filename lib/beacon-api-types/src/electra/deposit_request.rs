use unionlabs::primitives::{H256, H384, H768};

use crate::custom_types::Gwei;

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
    pub amount: Gwei,
    pub signature: H768,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    // TODO: Type?
    pub index: u64,
}
