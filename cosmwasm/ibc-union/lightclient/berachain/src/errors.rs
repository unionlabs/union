use ibc_union_light_client::IbcClientError;
use ics23::ibc_api::VerifyMembershipError;
use tendermint_light_client::{
    client::TendermintLightClient,
    errors::{
        IbcHeightTooLargeForTendermintHeight, InvalidChainId, InvalidHeaderError,
        InvalidHostTimestamp, MathOverflow, MerkleProofDecode, MigrateClientStoreError,
        NegativeTimestamp, RevisionNumberMismatch, TrustedValidatorsMismatch,
    },
};
use unionlabs::primitives::H256;

use crate::client::BerachainLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unimplemented")]
    Unimplemented,

    #[error(transparent)]
    NegativeTimestamp(#[from] NegativeTimestamp),

    #[error("invalid header")]
    InvalidHeader(#[from] InvalidHeaderError),

    #[error(transparent)]
    MathOverflow(#[from] MathOverflow),

    #[error(transparent)]
    MerkleProofDecode(#[from] MerkleProofDecode),

    #[error(transparent)]
    IbcHeightTooLargeForTendermintHeight(#[from] IbcHeightTooLargeForTendermintHeight),

    // NOTE: This is only emitted when it's not possible to parse the revision number from the chain id; perhaps make this more descriptive?
    #[error("invalid chain id {0}")]
    InvalidChainId(String),

    #[error(transparent)]
    ExecutionPayloadHeader(#[from] beacon_api_types::deneb::execution_payload_header::ssz::Error),

    #[error(transparent)]
    MigrateClientStore(#[from] MigrateClientStoreError),

    #[error(transparent)]
    TendermintVerify(#[from] tendermint_verifier::error::Error),

    #[error(transparent)]
    InvalidHostTimestamp(#[from] InvalidHostTimestamp),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("unable to verify execution header proof")]
    ExecutionHeaderVerify(#[source] ics23::ibc_api::VerifyMembershipError),

    #[error(transparent)]
    L1VerifyMembership(#[from] VerifyMembershipError),

    #[error(transparent)]
    VerifyStorage(#[from] evm_storage_verifier::error::Error),

    #[error(transparent)]
    VerifyMembership(#[from] ethereum_light_client::errors::Error),

    #[error("error while querying l1 state: {0}")]
    L1Error(#[from] IbcClientError<TendermintLightClient>),

    #[error("trusted validators don't match ({calculated} != {given})")]
    TrustedValidatorsMismatch { calculated: H256, given: H256 },

    #[error("revision number mismatch with trusted {trusted_revision_number} and the header {header_revision_number}")]
    RevisionNumberMismatch {
        trusted_revision_number: u64,
        header_revision_number: u64,
    },

    #[error("the signed header height {signed_height} must be more recent than the trusted height {trusted_height}")]
    SignedHeaderHeightMustBeMoreRecent {
        signed_height: u64,
        trusted_height: u64,
    },

    #[error("height {0} exceeds i64::MAX")]
    HeightTooLarge(u64),
}

// required for IbcClient trait
impl From<Error> for IbcClientError<BerachainLightClient> {
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
