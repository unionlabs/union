use serde::{Deserialize, Serialize};

use crate::{union::galois::prove_request::ProveRequest, Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PollRequest {
    pub request: ProveRequest,
}

impl Proto for PollRequest {
    type Proto = protos::union::galois::api::v1::PollRequest;
}

impl TypeUrl for protos::union::galois::api::v1::PollRequest {
    const TYPE_URL: &'static str = "/union.galois.api.v1.PollRequest";
}

impl From<PollRequest> for protos::union::galois::api::v1::PollRequest {
    fn from(value: PollRequest) -> Self {
        Self {
            request: Some(value.request.into()),
        }
    }
}
