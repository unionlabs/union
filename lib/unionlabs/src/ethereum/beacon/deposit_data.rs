use macros::model;
use ssz::Ssz;

use crate::hash::{H256, H384, H768};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#depositdata>
#[model]
#[derive(Ssz)]
pub struct DepositData {
    pub pubkey: H384,
    pub withdrawal_credentials: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub amount: u64,
    /// Signing over `DepositMessage`
    pub signature: H768,
}
