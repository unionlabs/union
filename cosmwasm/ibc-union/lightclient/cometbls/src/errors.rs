use cosmwasm_std::StdError;
use ibc_union_light_client::IbcClientError;
use ibc_union_spec::Timestamp;
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H256, encoding::HexUnprefixed},
};

use crate::{client::CometblsLightClient, zkp_verifier::ZkpVerifier};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum InvalidHeaderError {
    #[error(
        "signed header's height ({signed_height}) must be \
        greater than trusted height ({trusted_height})"
    )]
    SignedHeaderHeightMustBeMoreRecent {
        signed_height: u64,
        trusted_height: u64,
    },
    #[error(
        "signed header's timestamp ({signed_timestamp}) must be \
        greater than trusted timestamp ({trusted_timestamp})"
    )]
    SignedHeaderTimestampMustBeMoreRecent {
        signed_timestamp: Timestamp,
        trusted_timestamp: Timestamp,
    },
    #[error("header with timestamp ({0}) is expired")]
    HeaderExpired(Timestamp),
    #[error(
        "signed header timestamp ({signed_timestamp}) cannot \
        exceed the max clock drift ({max_clock_drift_timestamp})"
    )]
    SignedHeaderCannotExceedMaxClockDrift {
        signed_timestamp: Timestamp,
        max_clock_drift_timestamp: Timestamp,
    },
    #[error(
        "the validators hash ({actual}) doesn't match the trusted \
        validators hash ({expected}) for an adjacent block"
    )]
    InvalidValidatorsHash {
        expected: H256<HexUnprefixed>,
        actual: H256<HexUnprefixed>,
    },
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("math operation with overflow")]
    MathOverflow,

    #[error("unimplemented feature")]
    Unimplemented,

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

    #[error("header_b.height should be greater than or equal to header_a.height")]
    InvalidMisbehaviourHeaderSequence,

    #[error("given headers don't prove a misbehaviour")]
    MisbehaviourNotFound,
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

impl From<Error> for StdError {
    fn from(value: Error) -> Self {
        StdError::generic_err(value.to_string())
    }
}
