use macros::model;

use crate::{
    errors::{required, MissingField},
    union::galois::prove_response::{ProveResponse, TryFromProveResponseError},
};

#[model(proto(raw(protos::union::galois::api::v3::PollResponse), into, from))]
pub enum PollResponse {
    Pending,
    Failed(ProveRequestFailed),
    Done(ProveRequestDone),
}

#[model]
pub struct ProveRequestFailed {
    pub message: String,
}

#[model]
pub struct ProveRequestDone {
    pub response: ProveResponse,
}

impl From<PollResponse> for protos::union::galois::api::v3::PollResponse {
    fn from(value: PollResponse) -> Self {
        Self {
            result: Some(match value {
                PollResponse::Pending => {
                    protos::union::galois::api::v3::poll_response::Result::Pending(
                        protos::union::galois::api::v3::ProveRequestPending {},
                    )
                }
                PollResponse::Failed(failed) => {
                    protos::union::galois::api::v3::poll_response::Result::Failed(
                        protos::union::galois::api::v3::ProveRequestFailed {
                            message: failed.message,
                        },
                    )
                }
                PollResponse::Done(done) => {
                    protos::union::galois::api::v3::poll_response::Result::Done(
                        protos::union::galois::api::v3::ProveRequestDone {
                            response: Some(done.response.into()),
                        },
                    )
                }
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TryFromPollResponseError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid prove_response")]
    ProveResponse(#[source] TryFromProveResponseError),
}

impl TryFrom<protos::union::galois::api::v3::PollResponse> for PollResponse {
    type Error = TryFromPollResponseError;

    fn try_from(value: protos::union::galois::api::v3::PollResponse) -> Result<Self, Self::Error> {
        match required!(value.result)? {
            protos::union::galois::api::v3::poll_response::Result::Pending(_) => Ok(Self::Pending),
            protos::union::galois::api::v3::poll_response::Result::Failed(failed) => {
                Ok(Self::Failed(ProveRequestFailed {
                    message: failed.message,
                }))
            }
            protos::union::galois::api::v3::poll_response::Result::Done(done) => {
                Ok(Self::Done(ProveRequestDone {
                    response: required!(done.response)?
                        .try_into()
                        .map_err(TryFromPollResponseError::ProveResponse)?,
                }))
            }
        }
    }
}
