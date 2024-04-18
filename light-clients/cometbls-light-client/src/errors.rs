use cosmwasm_std::StdError;
use thiserror::Error as ThisError;
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    hash::H256,
    ibc::{core::client::height::Height, lightclients::cometbls::header::Header},
};

#[derive(ThisError, Debug, PartialEq)]
pub enum InvalidHeaderError {
    #[error("signed header's height ({signed_height}) must be greater than trusted height ({trusted_height})")]
    SignedHeaderHeightMustBeMoreRecent {
        signed_height: u64,
        trusted_height: u64,
    },
    #[error("signed header's timestamp ({signed_timestamp}) must be greater than trusted timestamp ({trusted_timestamp})")]
    SignedHeaderTimestampMustBeMoreRecent {
        signed_timestamp: u64,
        trusted_timestamp: u64,
    },
    #[error("header with timestamp ({0}) is expired")]
    HeaderExpired(u64),
    #[error("negative header timestamp seconds ({0})")]
    NegativeTimestamp(i64),
    #[error("negative header timestamp nanos ({0})")]
    NegativeTimestampNanos(i32),
    #[error("signed header timestamp ({signed_timestamp}) cannot exceed the max clock drift ({max_clock_drift})")]
    SignedHeaderCannotExceedMaxClockDrift {
        signed_timestamp: u64,
        max_clock_drift: u64,
    },
    #[error("commit hash ({commit_hash}) does not match with the signed header root ({signed_header_root})")]
    SignedHeaderMismatchWithCommitHash {
        commit_hash: H256,
        signed_header_root: H256,
    },
    #[error("the validators hash ({actual}) doesn't match the trusted validators hash ({expected}) for an adjacent block")]
    InvalidValidatorsHash { expected: H256, actual: H256 },
}

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("math operation with overflow")]
    MathOverflow,

    #[error("timestamp is negative ({0})")]
    NegativeTimestamp(i64),

    #[error("error while decoding proto ({reason})")]
    DecodeFromProto { reason: String },

    #[error("unimplemented feature")]
    Unimplemented,

    #[error("Unable to decode header: {0:?}")]
    HeaderDecode(DecodeErrorOf<Proto, Header>),

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

    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderError),

    #[error("Invalid ZKP: {0:?}")]
    InvalidZKP(cometbls_groth16_verifier::Error),

    #[error("Invalid sync committee")]
    InvalidSyncCommittee,

    #[error("Merkle root cannot be calculated")]
    UnableToCalculateMerkleRoot,

    #[error("No next sync committee")]
    NoNextSyncCommittee,

    #[error("Consensus state not found for {0}")]
    ConsensusStateNotFound(Height),

    #[error("Overflow happened during summing durations.")]
    DurationAdditionOverflow,

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

    #[error("Wasm client error: {0}")]
    Wasm(String),

    #[error("verify membership error: {0}")]
    VerifyMembership(#[from] ics23::ibc_api::VerifyMembershipError),

    #[error("substitute client is frozen")]
    SubstituteClientFrozen,

    #[error("forbidden fields have been changed during state migration")]
    MigrateFieldsChanged,

    #[error("the chain id cannot be more than 31 bytes long to fit in the bn254 scalar field")]
    InvalidChainID,
}

impl Error {
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

impl From<ics008_wasm_client::storage_utils::Error> for Error {
    fn from(error: ics008_wasm_client::storage_utils::Error) -> Self {
        match error {
            ics008_wasm_client::storage_utils::Error::ClientStateNotFound => {
                Error::ClientStateNotFound
            }
            ics008_wasm_client::storage_utils::Error::ClientStateDecode => Error::DecodeFromProto {
                reason: error.to_string(),
            },
            ics008_wasm_client::storage_utils::Error::ConsensusStateDecode => {
                Error::DecodeFromProto {
                    reason: error.to_string(),
                }
            }
        }
    }
}
