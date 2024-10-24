use serde::{Deserialize, Serialize};
use unionlabs::errors::{required, MissingField};

use crate::prove_response::{self, ProveResponse};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PollResponse {
    Pending,
    Failed(ProveRequestFailed),
    Done(ProveRequestDone),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProveRequestFailed {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
pub enum Error {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid prove_response")]
    ProveResponse(#[source] prove_response::Error),
}

impl TryFrom<protos::union::galois::api::v3::PollResponse> for PollResponse {
    type Error = Error;

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
                        .map_err(Error::ProveResponse)?,
                }))
            }
        }
    }
}
