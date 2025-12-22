use ethereum_light_client::client::EthereumLightClient;
use ibc_union_light_client::IbcClientError;

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

    #[error(transparent)]
    ParliaVerify(#[from] parlia_verifier::Error<CwContextError>),

    #[error(transparent)]
    EvmStorageVerify(#[from] evm_storage_verifier::error::Error),

    #[error(transparent)]
    Evm(#[from] ethereum_light_client::errors::Error),

    #[error(transparent)]
    EvmIbcClient(#[from] IbcClientError<EthereumLightClient>),
}

// required for IbcClient trait
impl From<Error> for IbcClientError<ParliaLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
