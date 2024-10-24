use cometbft_types::types::header::Header;
use serde::{Deserialize, Serialize};

use crate::{canonical_vote::CanonicalVote, validator_set_commit::ValidatorSetCommit};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
