use serde::{Deserialize, Serialize};

use crate::prove_request::ProveRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PollRequest {
    pub request: ProveRequest,
}

impl From<PollRequest> for protos::union::galois::api::v3::PollRequest {
    fn from(value: PollRequest) -> Self {
        Self {
            request: Some(value.request.into()),
        }
    }
}
