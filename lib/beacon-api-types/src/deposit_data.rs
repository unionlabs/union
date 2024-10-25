use unionlabs::{
    bls::{BlsPublicKey, BlsSignature},
    hash::H256,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepositData {
    pub pubkey: BlsPublicKey,
    pub withdrawal_credentials: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub amount: u64,
    /// Signing over `DepositMessage`
    pub signature: BlsSignature,
}
