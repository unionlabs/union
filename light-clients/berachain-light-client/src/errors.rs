use ethereum_light_client::errors::{CanonicalizeStoredValueError, StoredValueMismatch};
use ics008_wasm_client::IbcClientError;
use tendermint_light_client::errors::{
    IbcHeightTooLargeForTendermintHeight, InvalidChainId, InvalidHeaderError, InvalidHostTimestamp,
    MathOverflow, MerkleProofDecode, MigrateClientStoreError, NegativeTimestamp,
    RevisionNumberMismatch, TrustedValidatorsMismatch,
};
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    ibc::lightclients::{berachain, ethereum::storage_proof::StorageProof},
};

use crate::client::BerachainLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("unimplemented")]
    Unimplemented,

    #[error(transparent)]
    NegativeTimestamp(#[from] NegativeTimestamp),

    #[error("invalid header")]
    InvalidHeader(#[from] InvalidHeaderError),

    #[error(transparent)]
    CanonicalizeStoredValue(#[from] CanonicalizeStoredValueError),

    #[error(transparent)]
    StoredValueMismatch(#[from] StoredValueMismatch),

    #[error(transparent)]
    MathOverflow(#[from] MathOverflow),

    #[error(transparent)]
    MerkleProofDecode(#[from] MerkleProofDecode),

    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, berachain::client_state::ClientState>),

    #[error(transparent)]
    IbcHeightTooLargeForTendermintHeight(#[from] IbcHeightTooLargeForTendermintHeight),

    #[error(transparent)]
    RevisionNumberMismatch(#[from] RevisionNumberMismatch),

    // NOTE: This is only emitted when it's not possible to parse the revision number from the chain id; perhaps make this more descriptive?
    #[error(transparent)]
    InvalidChainId(#[from] InvalidChainId),

    #[error(transparent)]
    TrustedValidatorsMismatch(#[from] TrustedValidatorsMismatch),

    #[error("verify membership error")]
    VerifyMembership(#[from] ethereum_light_client::errors::Error),

    #[error(transparent)]
    MigrateClientStore(#[from] MigrateClientStoreError),

    #[error(transparent)]
    TendermintVerify(#[from] tendermint_verifier::error::Error),

    #[error(transparent)]
    InvalidHostTimestamp(#[from] InvalidHostTimestamp),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("unable to decode storage proof")]
    StorageProofDecode(#[source] DecodeErrorOf<Proto, StorageProof>),

    #[error("unable to verify execution header proof")]
    ExecutionHeaderVerify(#[source] ics23::ibc_api::VerifyMembershipError),

    #[error("unable to verify account storage root")]
    VerifyAccountStorageRoot(#[source] ethereum_verifier::error::Error),
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
