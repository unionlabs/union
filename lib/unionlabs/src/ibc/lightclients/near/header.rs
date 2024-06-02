use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::{hash::CryptoHash, types::BlockHeight};

use crate::near::types::{LightClientBlockView, MerklePath};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Header {
    pub new_state: LightClientBlockView,
    pub trusted_height: BlockHeight,
    pub prev_state_root_proof: MerklePath,
    pub prev_state_root: CryptoHash,
}
