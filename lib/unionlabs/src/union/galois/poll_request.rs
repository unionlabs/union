use macros::model;

use crate::union::galois::prove_request::ProveRequest;

#[model(proto(raw(protos::union::galois::api::v3::PollRequest), from))]
pub struct PollRequest {
    pub request: ProveRequest,
}

#[cfg(feature = "proto")]
impl From<PollRequest> for protos::union::galois::api::v3::PollRequest {
    fn from(value: PollRequest) -> Self {
        Self {
            request: Some(value.request.into()),
        }
    }
}
