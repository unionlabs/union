use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_proof::MerkleProof},
        lightclients::cometbls::{self, header::TryFromHeaderError},
    },
    TryFromProtoBytesError,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("math operation with overflow")]
    MathOverflow,

    #[error("unimplemented feature")]
    Unimplemented,

    #[error("unable to decode merkle proof")]
    MerkleProofDecode(#[source] DecodeErrorOf<Proto, MerkleProof>),

    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, cometbls::client_state::ClientState>),

    #[error("client not found")]
    ClientNotFound(String),

    #[error("Client state not found")]
    ClientStateNotFound,

    #[error("Consensus state not found for {0}")]
    ConsensusStateNotFound(Height),

    #[error("verify membership error: {0}")]
    VerifyMembership(#[from] ics23::ibc_api::VerifyMembershipError),

    #[error("substitute client is frozen")]
    SubstituteClientFrozen,

    #[error("forbidden fields have been changed during state migration")]
    MigrateFieldsChanged,

    #[error("the chain id cannot be more than 31 bytes long to fit in the bn254 scalar field")]
    InvalidChainId,

    #[error("invalid zkp length")]
    InvalidZKPLength,

    #[error("invalid height")]
    InvalidHeight,

    #[error("invalid timestamp")]
    InvalidTimestamp,

    #[error(
        "trusted validators hash ({expected:?}) does not match the given header's hash ({actual})"
    )]
    InvalidValidatorsHash { expected: H256, actual: H256 },

    #[error("signed header ({signed_timestamp}) exceeds the max clock drift ({max_clock_drift})")]
    SignedHeaderCannotExceedMaxClockDrift {
        signed_timestamp: u64,
        max_clock_drift: u64,
    },

    #[error("header with timestamp ({0}) is expired")]
    HeaderExpired(u64),

    #[error("signed header with timestamp ({signed_timestamp}) must be more recent than the trusted one ({trusted_timestamp})")]
    SignedHeaderTimestampMustBeMoreRecent {
        signed_timestamp: u64,
        trusted_timestamp: u64,
    },

    #[error("signed header with height ({signed_height}) must be more recent than the trusted one ({trusted_height})")]
    SignedHeaderHeightMustBeMoreRecent {
        signed_height: u64,
        trusted_height: u64,
    },

    #[error("header decode failed {0}")]
    HeaderDecode(#[from] TryFromProtoBytesError<TryFromHeaderError>),
}
