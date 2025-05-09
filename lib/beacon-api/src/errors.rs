use reqwest::StatusCode;
use serde_json::Value;

// REVIEW: Merge internal/not found with other?
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error")]
    Http(#[from] reqwest::Error),
    #[error("internal server error (raw response: {0})")]
    Internal(Value),
    #[error("not found (raw response: {0})")]
    NotFound(Value),
    #[error("json deserialization error")]
    Json(#[from] serde_json::Error),
    #[error("unknown error ({code}): {text}")]
    Other { code: StatusCode, text: String },
}
