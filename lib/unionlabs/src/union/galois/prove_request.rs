use macros::model;

use crate::{
    cometbls::types::canonical_vote::CanonicalVote, tendermint::types::header::Header,
    union::galois::validator_set_commit::ValidatorSetCommit,
};

#[model(proto(raw(protos::union::galois::api::v3::ProveRequest), from))]
pub struct ProveRequest {
    pub vote: CanonicalVote,
    pub untrusted_header: Header,
    pub trusted_commit: ValidatorSetCommit,
    pub untrusted_commit: ValidatorSetCommit,
}

impl From<ProveRequest> for protos::union::galois::api::v3::ProveRequest {
    fn from(value: ProveRequest) -> Self {
        Self {
            vote: Some(value.vote.into()),
            untrusted_header: Some(value.untrusted_header.into()),
            trusted_commit: Some(value.trusted_commit.into()),
            untrusted_commit: Some(value.untrusted_commit.into()),
        }
    }
}
