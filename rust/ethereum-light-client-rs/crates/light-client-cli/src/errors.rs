use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// rpc error: `{0}`
    RPCError(lodestar_rpc::errors::Error),
    /// io error: `{0}`
    IOError(std::io::Error),
    /// serde error: `{0}`
    SerdeError(serde_json::Error),
    /// verifier error: `{0}`
    VerifierError(ethereum_light_client_verifier::errors::Error),
    /// common error: `{0}`
    CommontError(ethereum_consensus::errors::Error),
    /// finalized header not found
    FinalizedHeaderNotFound,
    /// other error: `{description}`
    Other { description: String },
}

impl std::error::Error for Error {}

impl From<lodestar_rpc::errors::Error> for Error {
    fn from(value: lodestar_rpc::errors::Error) -> Self {
        Self::RPCError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeError(value)
    }
}

impl From<ethereum_light_client_verifier::errors::Error> for Error {
    fn from(value: ethereum_light_client_verifier::errors::Error) -> Self {
        Self::VerifierError(value)
    }
}

impl From<ethereum_consensus::errors::Error> for Error {
    fn from(value: ethereum_consensus::errors::Error) -> Self {
        Self::CommontError(value)
    }
}
