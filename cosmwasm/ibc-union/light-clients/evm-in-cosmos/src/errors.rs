use cosmwasm_std::StdError;
use ibc_union_light_client::IbcClientError;
use unionlabs::ibc::core::client::height::Height;

use crate::client::EvmInCosmosLightClient;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unimplemented feature")]
    Unimplemented,

    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("verify l2 membership error")]
    VerifyL2Membership(#[from] ics23::ibc_api::VerifyMembershipError),

    #[error(transparent)]
    EthereumLightClient(#[from] ethereum_light_client::errors::Error),
}

impl From<Error> for IbcClientError<EvmInCosmosLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}

impl From<Error> for StdError {
    fn from(value: Error) -> Self {
        StdError::generic_err(value.to_string())
    }
}
