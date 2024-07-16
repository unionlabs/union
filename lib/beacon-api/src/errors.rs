use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

// NOTE: Some of these are specific to lodestar
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error")]
    Http(#[from] reqwest::Error),
    #[error("internal error")]
    Internal(#[from] InternalServerError),
    #[error("not found")]
    NotFound(#[from] NotFoundError),
    #[error("json deserialization error")]
    Json(#[from] serde_json::Error),
    #[error("unknown error ({code}): {text}")]
    Other { code: StatusCode, text: String },
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
#[error("{status_code} {error}: {message}")]
pub struct NotFoundError {
    #[serde(rename = "statusCode")]
    pub status_code: u64,
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
#[error("{status_code} {error}: {message}")]
pub struct InternalServerError {
    #[serde(rename = "statusCode")]
    pub status_code: u64,
    pub error: String,
    pub message: String,
}
