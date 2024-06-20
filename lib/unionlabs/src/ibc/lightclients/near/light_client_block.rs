use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::hash::CryptoHash;

use super::{block_header_inner::BlockHeaderInnerLiteView, validator_stake::ValidatorStakeView};
use crate::near::types::Signature;

#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    BorshDeserialize,
    BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct LightClientBlockView {
    pub prev_block_hash: CryptoHash,
    pub next_block_inner_hash: CryptoHash,
    pub inner_lite: BlockHeaderInnerLiteView,
    pub inner_rest_hash: CryptoHash,
    pub next_bps: Option<Vec<ValidatorStakeView>>,
    pub approvals_after_next: Vec<Option<Box<Signature>>>,
}
