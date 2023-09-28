use std::fmt::Display;

use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    Internal(InternalServerError),
    NotFound(NotFoundError),
    Json(serde_json::Error),
    Other { code: StatusCode, text: String },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NotFoundError {
    #[serde(rename = "statusCode")]
    pub status_code: u64,
    pub error: String,
    pub message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InternalServerError {
    #[serde(rename = "statusCode")]
    pub status_code: u64,
    pub error: String,
    pub message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(err) => write!(f, "Http error: {err}"),
            Self::Internal(err) => write!(f, "Internal server error: {err:#?}"),
            Error::NotFound(err) => write!(f, "Not found: {err:#?}"),
            Self::Json(err) => write!(f, "Deserialization error: {err}"),
            Self::Other { code, text } => write!(f, "Unknown error (code {code}): {text}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl std::error::Error for Error {}
