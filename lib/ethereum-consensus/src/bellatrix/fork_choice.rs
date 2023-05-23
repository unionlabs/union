use crate::primitives::Hash32;
use ssz_rs::prelude::*;

#[derive(Default, Debug, SimpleSerialize, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PowBlock {
    block_hash: Hash32,
    parent_hash: Hash32,
    total_difficulty: U256,
}
