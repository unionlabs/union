use ics008_wasm_client::IbcClientError;
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_proof::MerkleProof},
        lightclients::{cometbls::header::Header, tendermint},
    },
};

use crate::client::TendermintLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
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
    #[error("negative header timestamp ({0})")]
    NegativeTimestamp(i64),
    #[error("signed header timestamp ({signed_timestamp}) cannot exceed the max clock drift ({max_clock_drift})")]
    SignedHeaderCannotExceedMaxClockDrift {
        signed_timestamp: u64,
        max_clock_drift: u64,
    },
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("math operation with overflow")]
    MathOverflow,

    #[error("timestamp is negative ({0})")]
    NegativeTimestamp(i64),

    #[error("unimplemented feature")]
    // TODO: Remove this from this variant
    Unimplemented,

    #[error("unable to decode header")]
    HeaderDecode(#[source] DecodeErrorOf<Proto, Header>),

    #[error("unable to decode merkle proof")]
    MerkleProofDecode(#[source] DecodeErrorOf<Proto, MerkleProof>),

    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, tendermint::client_state::ClientState>),

    #[error("the ibc height.revision_height does not fit in an i64 ({0})")]
    IbcHeightTooLargeForTendermintHeight(u64),

    #[error("trusted revision number ({trusted_revision_number}) does not match the header ({header_revision_number})")]
    RevisionNumberMismatch {
        trusted_revision_number: u64,
        header_revision_number: u64,
    },

    #[error("invalid header")]
    InvalidHeader(#[from] InvalidHeaderError),

    #[error("consensus state not found for {0}")]
    // TODO: Move this variant into IbcClientError
    ConsensusStateNotFound(Height),

    // NOTE: This is only emitted when it's not possible to parse the revision number from the chain id; perhaps make this more descriptive?
    #[error("invalid chain id ({0})")]
    InvalidChainId(String),

    #[error("trusted validators hash ({0}) does not match the saved one ({1})")]
    TrustedValidatorsMismatch(H256, H256),

    #[error("verify membership error: {0}")]
    VerifyMembership(#[from] ics23::ibc_api::VerifyMembershipError),

    #[error(transparent)]
    TendermintVerify(#[from] tendermint_verifier::error::Error),

    #[error("invalid timestamp from the host ({0})")]
    InvalidHostTimestamp(cosmwasm_std::Timestamp),

    #[error("substitute client is frozen")]
    SubstituteClientFrozen,

    #[error("forbidden fields have been changed during state migration")]
    MigrateFieldsChanged,
}

// required for IbcClient trait
impl From<Error> for IbcClientError<TendermintLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

// convenience
impl From<InvalidHeaderError> for IbcClientError<TendermintLightClient> {
    fn from(value: InvalidHeaderError) -> Self {
        IbcClientError::ClientSpecific(Error::InvalidHeader(value))
    }
}

// would be nice, but both foreign types :(
// impl From<ics23::ibc_api::VerifyMembershipError> for IbcClientError<TendermintLightClient> {
//     fn from(value: ics23::ibc_api::VerifyMembershipError) -> Self {
//         IbcClientError::ClientSpecific(Error::VerifyMembership(value))
//     }
// }
