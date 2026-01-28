use cosmwasm_std::StdError;
use ibc_union_light_client::IbcClientError;

use crate::client::ProofLensLightClient;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unimplemented feature")]
    Unimplemented,
}

impl From<Error> for IbcClientError<ProofLensLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

impl From<Error> for StdError {
    fn from(value: Error) -> Self {
        StdError::generic_err(value.to_string())
    }
}
