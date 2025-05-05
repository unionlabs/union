use ibc_union_light_client::IbcClientError;
use tendermint_light_client_types::{ClientState, Header};
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    ibc::core::commitment::merkle_proof::MerkleProof,
    primitives::H256,
};

use crate::client::TendermintLightClient;

#[derive(Debug, thiserror::Error)]
// TODO: Use an error reporter at the top level of ics008-wasm-client so we don't have to include the sources manually in the display impl
pub enum Error {
    #[error(transparent)]
    MathOverflow(#[from] MathOverflow),

    #[error(transparent)]
    NegativeTimestamp(#[from] NegativeTimestamp),

    #[error("unimplemented feature")]
    // TODO: Remove this from this variant
    Unimplemented,

    #[error("unable to decode header")]
    HeaderDecode(#[source] DecodeErrorOf<Proto, Header>),

    #[error(transparent)]
    MerkleProofDecode(#[from] MerkleProofDecode),

    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, ClientState>),

    #[error(transparent)]
    IbcHeightTooLargeForTendermintHeight(#[from] IbcHeightTooLargeForTendermintHeight),

    #[error(transparent)]
    RevisionNumberMismatch(#[from] RevisionNumberMismatch),

    #[error("invalid header")]
    InvalidHeader(#[from] InvalidHeaderError),

    // NOTE: This is only emitted when it's not possible to parse the revision number from the chain id; perhaps make this more descriptive?
    #[error(transparent)]
    InvalidChainId(#[from] InvalidChainId),

    #[error(transparent)]
    TrustedValidatorsMismatch(#[from] TrustedValidatorsMismatch),

    #[error("verify membership error")]
    VerifyMembership(#[from] ics23::ibc_api::VerifyMembershipError),

    #[error(transparent)]
    MigrateClientStore(#[from] MigrateClientStoreError),

    #[error(transparent)]
    TendermintVerify(#[from] tendermint_verifier::error::Error),

    #[error(transparent)]
    InvalidHostTimestamp(#[from] InvalidHostTimestamp),

    #[error("invalid or empty validator set, supported keys are: bls12381 and ed25519")]
    InvalidValidatorSet,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
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

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MigrateClientStoreError {
    #[error("substitute client is frozen")]
    SubstituteClientFrozen,

    #[error("forbidden fields have been changed during state migration")]
    MigrateFieldsChanged,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("trusted validators hash ({0}) does not match the saved one ({1})")]
pub struct TrustedValidatorsMismatch(pub H256, pub H256);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("trusted revision number ({trusted_revision_number}) does not match the header ({header_revision_number})")]
pub struct RevisionNumberMismatch {
    pub trusted_revision_number: u64,
    pub header_revision_number: u64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("timestamp is negative ({0})")]
pub struct NegativeTimestamp(pub i64);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("unable to decode merkle proof")]
pub struct MerkleProofDecode(#[source] pub DecodeErrorOf<Proto, MerkleProof>);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("the ibc height.revision_height does not fit in an i64 ({0})")]
pub struct IbcHeightTooLargeForTendermintHeight(pub u64);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid timestamp from the host ({0})")]
pub struct InvalidHostTimestamp(pub cosmwasm_std::Timestamp);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid chain id ({0})")]
pub struct InvalidChainId(pub String);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("math operation with overflow")]
pub struct MathOverflow;

// required for IbcClient trait
impl From<Error> for IbcClientError<TendermintLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

// would be nice, but both foreign types :(
// impl From<ics23::ibc_api::VerifyMembershipError> for IbcClientError<TendermintLightClient> {
//     fn from(value: ics23::ibc_api::VerifyMembershipError) -> Self {
//         IbcClientError::ClientSpecific(Error::VerifyMembership(value))
//     }
// }
