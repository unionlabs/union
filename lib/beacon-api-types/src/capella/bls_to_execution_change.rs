use unionlabs::primitives::{H160, H384};

use crate::custom_types::ValidatorIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ssz", derive(ssz::Ssz))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlsToExecutionChange {
    pub validator_index: ValidatorIndex,
    pub from_bls_pubkey: H384,
    pub to_execution_address: H160,
}
