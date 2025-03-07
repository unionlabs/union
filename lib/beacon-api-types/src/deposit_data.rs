use unionlabs::primitives::{H256, H384, H768};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepositData {
    pub pubkey: H384,
    pub withdrawal_credentials: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub amount: u64,
    /// Signing over `DepositMessage`
    pub signature: H768,
}
