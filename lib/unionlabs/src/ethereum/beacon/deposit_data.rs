use macros::model;

use crate::hash::{H256, H384, H768};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#depositdata>
#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct DepositData {
    pub pubkey: H384,
    pub withdrawal_credentials: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub amount: u64,
    /// Signing over `DepositMessage`
    pub signature: H768,
}
