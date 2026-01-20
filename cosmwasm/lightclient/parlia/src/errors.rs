use ibc_union_light_client::IbcClientError;
use unionlabs::primitives::FixedBytesError;

use crate::client::{CwContextError, ParliaLightClient};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unimplemented")]
    Unimplemented,

    #[error("no initial valset provided on client creation")]
    NoInitialValset,

    #[error("misbehaviour headers were not for the same height")]
    MisbehaviourHeadersNotForSameHeight,

    #[error("misbehaviour headers were exactly equal")]
    MisbehaviourHeadersMustBeDifferent,

    #[error("invalid storage proof key")]
    InvalidKey(FixedBytesError),

    #[error("invalid storage proof value")]
    InvalidValue(FixedBytesError),

    #[error(transparent)]
    ParliaVerify(#[from] parlia_verifier::Error<CwContextError>),

    #[error("invalid account storage root")]
    VerifyAccountStorageRoot(#[source] evm_storage_verifier::error::Error),

    #[error("invalid storage proof")]
    VerifyStorageProof(#[source] evm_storage_verifier::error::Error),

    #[error("invalid storage absence proof")]
    VerifyStorageAbsence(#[source] evm_storage_verifier::error::Error),
}

// required for IbcClient trait
impl From<Error> for IbcClientError<ParliaLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
