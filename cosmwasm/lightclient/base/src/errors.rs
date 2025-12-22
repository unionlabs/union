use ethereum_light_client::client::EthereumLightClient;
use ibc_union_light_client::IbcClientError;

use crate::client::BaseLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unimplemented")]
    Unimplemented,

    #[error(transparent)]
    Verify(#[from] base_verifier::Error),

    #[error(transparent)]
    Evm(#[from] ethereum_light_client::errors::Error),

    #[error(transparent)]
    EvmIbcClient(#[from] IbcClientError<EthereumLightClient>),
}

// required for IbcClient trait
impl From<Error> for IbcClientError<BaseLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
