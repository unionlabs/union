use unionlabs::primitives::{H160, H384};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlsToExecutionChange {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub validator_index: u64,
    pub from_bls_pubkey: H384,
    pub to_execution_address: H160,
}
