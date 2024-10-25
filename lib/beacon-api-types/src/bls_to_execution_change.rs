use unionlabs::{bls::BlsPublicKey, hash::H160};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlsToExecutionChange {
    #[serde(with = "::serde_utils::string")]
    pub validator_index: u64,
    pub from_bls_pubkey: BlsPublicKey,
    pub to_execution_address: H160,
}
