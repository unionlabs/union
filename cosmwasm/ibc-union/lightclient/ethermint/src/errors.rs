use ibc_union_light_client::IbcClientError;
use unionlabs::primitives::FixedBytesError;

use crate::client::EthermintLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tendermint light client error")]
    Tendermint(#[source] tendermint_light_client::errors::Error),
    #[error("invalid key")]
    InvalidKey(#[source] FixedBytesError),
}

impl<T: Into<tendermint_light_client::errors::Error>> From<T> for Error {
    fn from(value: T) -> Self {
        Error::Tendermint(value.into())
    }
}

// required for IbcClient trait
impl From<Error> for IbcClientError<EthermintLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
