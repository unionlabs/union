use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{
    cometbls::types::canonical_vote::CanonicalVote,
    union::galois::validator_set_commit::ValidatorSetCommit,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::union::galois::api::v2::ProveRequest, from)]
pub struct ProveRequest {
    pub vote: CanonicalVote,
    pub trusted_commit: ValidatorSetCommit,
    pub untrusted_commit: ValidatorSetCommit,
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
