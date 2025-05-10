use crate::{
    checkpoint_summary::{CheckpointSummary, ExecutionDigests},
    crypto::AuthorityStrongQuorumSignInfo,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub trusted_height: u64,
    pub checkpoint_summary: CheckpointSummary,
    pub sign_info: AuthorityStrongQuorumSignInfo,
    pub transactions: Vec<ExecutionDigests>,
}
