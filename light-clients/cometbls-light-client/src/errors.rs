use cosmwasm_std::StdError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),

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

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid height")]
    InvalidHeight,

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Invalid ZKP")]
    InvalidZKP,

    #[error("Invalid sync committee")]
    InvalidSyncCommittee,

    #[error("No next sync committee")]
    NoNextSyncCommittee,

    #[error("Consensus state not found for {0}-{1}")]
    // REVIEW: Why not just use `Height` directly?
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

    #[error("Invalid path {0}")]
    InvalidPath(String),

    #[error("Invalid membership value")]
    InvalidValue,

    #[error("Invalid commitment key. Expected {0}, got {1}.")]
    InvalidCommitmentKey(String, String),

    #[error("Missing field in the protobuf encoded data")]
    MissingProtoField,

    #[error("Client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("Proof is empty")]
    EmptyProof,

    #[error("Batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("Expected value: '{0}' and stored value '{1}' doesn't match")]
    ExpectedAndStoredValueMismatch(String, String),

    #[error("Custom query: {0}")]
    CustomQuery(String),
}

impl Error {
    pub fn decode<S: Into<String>>(s: S) -> Error {
        Error::DecodeError(s.into())
    }

    pub fn invalid_public_key<S: ToString>(s: S) -> Error {
        Error::InvalidPublicKey(s.to_string())
    }

    pub fn invalid_commitment_key<B1: AsRef<[u8]>, B2: AsRef<[u8]>>(
        expected: B1,
        got: B2,
    ) -> Error {
        Error::InvalidCommitmentKey(hex::encode(expected), hex::encode(got))
    }

    pub fn stored_value_mismatch<B1: AsRef<[u8]>, B2: AsRef<[u8]>>(expected: B1, got: B2) -> Error {
        Error::ExpectedAndStoredValueMismatch(hex::encode(expected), hex::encode(got))
    }

    pub fn custom_query<S: ToString>(s: S) -> Error {
        Error::CustomQuery(s.to_string())
    }
}

impl From<wasm_light_client_types::Error> for Error {
    fn from(error: wasm_light_client_types::Error) -> Self {
        match error {
            wasm_light_client_types::Error::Decode(e) => Error::DecodeError(e),
        }
    }
}
