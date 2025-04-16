use unionlabs::primitives::{H256, H384, H768};

use crate::custom_types::Gwei;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct DepositData {
    pub pubkey: H384,
    pub withdrawal_credentials: H256,
    pub amount: Gwei,
    /// Signing over `DepositMessage`
    pub signature: H768,
}
