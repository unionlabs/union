use ics008_wasm_client::IbcClientError;
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_proof::MerkleProof},
        lightclients::cometbls,
    },
};

use crate::{client::CometblsLightClient, zkp_verifier::ZkpVerifier};

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
    #[error("signed header timestamp ({signed_timestamp}) cannot exceed the max clock drift ({max_clock_drift})")]
    SignedHeaderCannotExceedMaxClockDrift {
        signed_timestamp: u64,
        max_clock_drift: u64,
    },
    #[error("the validators hash ({actual}) doesn't match the trusted validators hash ({expected}) for an adjacent block")]
    InvalidValidatorsHash { expected: H256, actual: H256 },
}

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

    #[error("Client state not found")]
    ClientStateNotFound,

    #[error(transparent)]
    InvalidHeader(#[from] InvalidHeaderError),

    #[error("Invalid ZKP: {0:?}")]
    InvalidZKP(cometbls_groth16_verifier::Error),

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
}

// required for IbcClient trait
impl<T: ZkpVerifier> From<Error> for IbcClientError<CometblsLightClient<T>> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

// convenience
impl<T: ZkpVerifier> From<InvalidHeaderError> for IbcClientError<CometblsLightClient<T>> {
    fn from(value: InvalidHeaderError) -> Self {
        IbcClientError::ClientSpecific(Error::InvalidHeader(value))
    }
}
