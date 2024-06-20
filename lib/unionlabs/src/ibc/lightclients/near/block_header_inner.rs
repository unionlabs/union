use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::{
    hash::CryptoHash,
    types::{BlockHeight, MerkleHash},
};

use crate::near::types::EpochId;

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
pub struct BlockHeaderInnerLiteView {
    pub height: BlockHeight,
    pub epoch_id: CryptoHash,
    pub next_epoch_id: CryptoHash,
    pub prev_state_root: CryptoHash,
    pub outcome_root: CryptoHash,
    /// Legacy json number. Should not be used.
    pub timestamp: u64,
    // TODO(aeryz): #[serde(with = "dec_format")]
    pub timestamp_nanosec: u64,
    pub next_bp_hash: CryptoHash,
    pub block_merkle_root: CryptoHash,
}

#[derive(BorshSerialize, BorshDeserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct BlockHeaderInnerLite {
    /// Height of this block.
    pub height: BlockHeight,
    /// Epoch start hash of this block's epoch.
    /// Used for retrieving validator information
    pub epoch_id: EpochId,
    pub next_epoch_id: EpochId,
    /// Root hash of the state at the previous block.
    pub prev_state_root: MerkleHash,
    /// Root of the outcomes of transactions and receipts from the previous chunks.
    pub prev_outcome_root: MerkleHash,
    /// Timestamp at which the block was built (number of non-leap-nanoseconds since January 1, 1970 0:00:00 UTC).
    pub timestamp: u64,
    /// Hash of the next epoch block producers set
    pub next_bp_hash: CryptoHash,
    /// Merkle root of block hashes up to the current block.
    pub block_merkle_root: CryptoHash,
}

impl From<BlockHeaderInnerLiteView> for BlockHeaderInnerLite {
    fn from(value: BlockHeaderInnerLiteView) -> Self {
        Self {
            height: value.height,
            epoch_id: EpochId(value.epoch_id),
            next_epoch_id: EpochId(value.next_epoch_id),
            prev_state_root: value.prev_state_root,
            prev_outcome_root: value.outcome_root,
            timestamp: value.timestamp,
            next_bp_hash: value.next_bp_hash,
            block_merkle_root: value.block_merkle_root,
        }
    }
}
