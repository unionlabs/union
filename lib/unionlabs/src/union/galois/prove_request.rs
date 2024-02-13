use serde::{Deserialize, Serialize};

use crate::{
    cometbls::types::canonical_vote::CanonicalVote,
    union::galois::validator_set_commit::ValidatorSetCommit, Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct ProveRequest {
    pub vote: CanonicalVote,
    pub trusted_commit: ValidatorSetCommit,
    pub untrusted_commit: ValidatorSetCommit,
}

impl Proto for ProveRequest {
    type Proto = protos::union::galois::api::v2::ProveRequest;
}

impl TypeUrl for protos::union::galois::api::v2::ProveRequest {
    const TYPE_URL: &'static str = "/union.galois.api.v2.ProveRequest";
}

impl From<ProveRequest> for protos::union::galois::api::v2::ProveRequest {
    fn from(value: ProveRequest) -> Self {
        Self {
            vote: Some(value.vote.into()),
            trusted_commit: Some(value.trusted_commit.into()),
            untrusted_commit: Some(value.untrusted_commit.into()),
        }
    }
}
