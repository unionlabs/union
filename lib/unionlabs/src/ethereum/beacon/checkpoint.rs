use macros::model;
use ssz::Ssz;

use crate::hash::H256;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#checkpoint>
#[model]
#[derive(Ssz)]
pub struct Checkpoint {
    #[serde(with = "::serde_utils::string")]
    pub epoch: u64,
    pub root: H256,
}
