use macros::model;
use ssz::Ssz;

use crate::hash::{hash_v2::Hash, H160};

#[model]
#[derive(Ssz)]
pub struct BlsToExecutionChange {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub validator_index: u64,
    pub from_bls_pubkey: Hash<48>,
    pub to_execution_address: H160,
}
