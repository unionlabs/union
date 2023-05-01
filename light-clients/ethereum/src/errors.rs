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

    #[error("Decode error")]
    DecodeError,

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
}
