use cosmwasm_std::Addr;
use ethereum_light_client::client::EthereumLightClient;
use ibc_union_light_client::IbcClientError;
use unionlabs::{ibc::core::client::height::Height, primitives::H256};

use crate::client::MptTrustedLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // REVIEW: Move this variant to IbcClientError?
    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("proof is empty")]
    EmptyProof,

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("the operation has not been implemented yet")]
    Unimplemented,

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
