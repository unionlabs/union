use cosmwasm_std::StdError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    /// this is needed so we can use `bucket.load(...)?` and have it auto-converted to the custom error
    Std(#[from] StdError),
    // this is whatever we want
    #[error("Unimplemented")]
    Unimplemented,

    #[error("Decode error: {0}")]
    DecodeError(String),

    #[error("Unknown type url")]
    UnknownTypeUrl,

    #[error("Client state not found")]
    ClientStateNotFound,

    #[error("Invalid proof format")]
    InvalidProofFormat,

    #[error("Invalid client id")]
    InvalidClientId,

    #[error("Invalid client id")]
    InvalidPublicKey,

    #[error("Invalid height")]
    InvalidHeight,

    #[error("Invalid sync committee")]
    InvalidSyncCommittee,

    #[error("No next sync committee")]
    NoNextSyncCommittee,

    #[error("Consensus state not found for {0}-{1}")]
    ConsensusStateNotFound(u64, u64),

    #[error("Timestamp not set")]
    TimestampNotSet,

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("Unexpected timestamp: Expected timestamp {0}, got {1}")]
    UnexpectedTimestamp(u64, u64),

    #[error("Future period")]
    FuturePeriod,

    #[error("Cannot generate proof")]
    CannotGenerateProof,

    #[error("Invalid chain version")]
    InvalidChainVersion,
}

impl Error {
    pub fn decode<S: Into<String>>(s: S) -> Error {
        Error::DecodeError(s.into())
    }
}
