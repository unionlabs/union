use cosmwasm_std::Addr;
use ethereum_light_client::client::EthereumLightClient;
use ibc_union_light_client::IbcClientError;

use crate::client::MptTrustedLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("misbehaviour logic is not needed in a trusted setup")]
    NoMisbehaviourInTrustedClient,

    #[error("the caller {0} is not authorized to update this client")]
    Unauthorized(Addr),

    #[error(transparent)]
    EvmIbcClient(#[from] IbcClientError<EthereumLightClient>),

    #[error("invalid contract address proof")]
    InvalidContractAddressProof(#[source] evm_storage_verifier::error::Error),

    #[error(transparent)]
    Evm(#[from] ethereum_light_client::errors::Error),
}

impl From<Error> for IbcClientError<MptTrustedLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
