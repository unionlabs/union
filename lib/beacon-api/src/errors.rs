use std::fmt::Display;

use reqwest::StatusCode;

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    Internal(InternalServerError),
    Json(serde_json::Error),
    Other { code: StatusCode, text: String },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InternalServerError {
    #[serde(rename = "statusCode")]
    status_code: u64,
    error: String,
    message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(err) => write!(f, "`{err}`"),
            Self::Internal(err) => write!(f, "{err:#?}"),
            Self::Json(err) => write!(f, "{err}"),
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
