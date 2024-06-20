use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::{
    hash::CryptoHash,
    types::{BlockHeight, MerkleHash},
};

use crate::{errors::MissingField, near::types::EpochId};

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

impl From<BlockHeaderInnerLiteView>
    for protos::union::ibc::lightclients::near::v1::BlockHeaderInnerLiteView
{
    fn from(value: BlockHeaderInnerLiteView) -> Self {
        Self {
            height: value.height,
            epoch_id: value.epoch_id.into(),
            next_epoch_id: value.next_epoch_id.into(),
            prev_state_root: value.prev_state_root.into(),
            outcome_root: value.outcome_root.into(),
            timestamp: value.timestamp,
            timestamp_nanosec: value.timestamp_nanosec,
            next_bp_hash: value.next_bp_hash.into(),
            block_merkle_root: value.block_merkle_root.into(),
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromBlockHeaderInnerLiteViewError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid epoch id")]
    EpochId,
    #[error("invalid next epoch id")]
    NextEpochId,
    #[error("invalid prev state root")]
    PrevStateRoot,
    #[error("invalid outcome root")]
    OutcomeRoot,
    #[error("next bp hash")]
    NextBpHash,
    #[error("block merkle root")]
    BlockMerkleRoot,
}

impl TryFrom<protos::union::ibc::lightclients::near::v1::BlockHeaderInnerLiteView>
    for BlockHeaderInnerLiteView
{
    type Error = TryFromBlockHeaderInnerLiteViewError;

    fn try_from(
        value: protos::union::ibc::lightclients::near::v1::BlockHeaderInnerLiteView,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            height: value.height,
            epoch_id: value
                .epoch_id
                .as_slice()
                .try_into()
                .map_err(|_| TryFromBlockHeaderInnerLiteViewError::EpochId)?,
            next_epoch_id: value
                .next_epoch_id
                .as_slice()
                .try_into()
                .map_err(|_| TryFromBlockHeaderInnerLiteViewError::NextEpochId)?,
            prev_state_root: value
                .prev_state_root
                .as_slice()
                .try_into()
                .map_err(|_| TryFromBlockHeaderInnerLiteViewError::PrevStateRoot)?,
            outcome_root: value
                .outcome_root
                .as_slice()
                .try_into()
                .map_err(|_| TryFromBlockHeaderInnerLiteViewError::OutcomeRoot)?,
            timestamp: value.timestamp,
            timestamp_nanosec: value.timestamp_nanosec,
            next_bp_hash: value
                .next_bp_hash
                .as_slice()
                .try_into()
                .map_err(|_| TryFromBlockHeaderInnerLiteViewError::NextBpHash)?,
            block_merkle_root: value
                .block_merkle_root
                .as_slice()
                .try_into()
                .map_err(|_| TryFromBlockHeaderInnerLiteViewError::BlockMerkleRoot)?,
        })
    }
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
