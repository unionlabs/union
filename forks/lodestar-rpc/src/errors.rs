use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// http error: `{0:?}`
    HTTPError(reqwest::Error),
    /// RPC internal server error: `{0}`
    RPCInternalServerError(String),
    /// json decode error: `{0}`
    JSONDecodeError(serde_json::Error),
    /// other error: `{description}`
    Other { description: String },
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::HTTPError(value)
    }
}

impl std::error::Error for Error {}
