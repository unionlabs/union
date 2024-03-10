use macros::model;
use serde::{Deserialize, Serialize};

use crate::union::galois::prove_request::ProveRequest;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[model(proto(raw(protos::union::galois::api::v2::PollRequest), from))]
pub struct PollRequest {
    pub request: ProveRequest,
}

impl From<PollRequest> for protos::union::galois::api::v2::PollRequest {
    fn from(value: PollRequest) -> Self {
        Self {
            request: Some(value.request.into()),
        }
    }
}
