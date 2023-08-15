use crate::{
    tendermint::types::canonical_vote::CanonicalVote,
    union::galois::validator_set_commit::ValidatorSetCommit,
};

#[derive(Clone, PartialEq)]
pub struct ProveRequest {
    pub vote: CanonicalVote,
    pub trusted_commit: ValidatorSetCommit,
    pub untrusted_commit: ValidatorSetCommit,
}
