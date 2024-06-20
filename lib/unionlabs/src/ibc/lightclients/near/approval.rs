use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::{hash::CryptoHash, types::BlockHeight};

/// The part of the block approval that is different for endorsements and skips
#[derive(BorshSerialize, BorshDeserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApprovalInner {
    Endorsement(CryptoHash),
    Skip(BlockHeight),
}
