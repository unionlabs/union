use macros::model;
use ssz::Ssz;

use crate::bls::{BlsPublicKey, BlsSignature};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#depositdata>
#[model]
#[derive(Ssz)]
pub struct DepositData {
    pub pubkey: BlsPublicKey,
    pub withdrawal_credentials: [u8; 32],
    #[serde(with = "::serde_utils::string")]
    pub amount: u64,
    /// Signing over `DepositMessage`
    pub signature: BlsSignature,
}
