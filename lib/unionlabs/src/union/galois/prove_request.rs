use serde::{Deserialize, Serialize};

use crate::{
    tendermint::types::canonical_vote::CanonicalVote,
    union::galois::validator_set_commit::ValidatorSetCommit, Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProveRequest {
    pub vote: CanonicalVote,
    pub trusted_commit: ValidatorSetCommit,
    pub untrusted_commit: ValidatorSetCommit,
}

impl Proto for ProveRequest {
    type Proto = protos::union::galois::api::v1::ProveRequest;
}

impl TypeUrl for protos::union::galois::api::v1::ProveRequest {
    const TYPE_URL: &'static str = "/union.galois.api.v1.ProveRequest";
}

impl From<ProveRequest> for protos::union::galois::api::v1::ProveRequest {
    fn from(value: ProveRequest) -> Self {
        Self {
            vote: Some(value.vote.into()),
            trusted_commit: Some(value.trusted_commit.into()),
            untrusted_commit: Some(value.untrusted_commit.into()),
        }
    }
}
