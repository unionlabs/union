use macros::model;
use ssz::Ssz;

use crate::{bls::BlsPublicKey, hash::H160};

#[model]
#[derive(Ssz)]
pub struct BlsToExecutionChange {
    #[serde(with = "::serde_utils::string")]
    pub validator_index: u64,
    pub from_bls_pubkey: BlsPublicKey,
    pub to_execution_address: H160,
}
