use crate::tendermint::types::{block_id::BlockId, commit_sig::CommitSig};

#[derive(Clone, PartialEq)]
pub struct Commit {
    pub height: u32,
    pub round: u16,
    pub block_id: BlockId,
    pub signatures: Vec<CommitSig>,
}
