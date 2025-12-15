use ethereum_light_client::client::EthereumLightClient;
use ibc_union_light_client::IbcClientError;
use unionlabs::primitives::Bytes;

use crate::client::StarknetLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unimplemented")]
    Unimplemented,

    #[error(transparent)]
    Verify(#[from] starknet_verifier::Error),

    #[error(transparent)]
    Proof(#[from] starknet_storage_verifier::Error),

    #[error(transparent)]
    EthereumLightClient(#[from] IbcClientError<EthereumLightClient>),

    #[error("invalid proof")]
    InvalidProof,

    #[error("invalid proof key: {0}")]
    InvalidProofKey(Bytes),

    #[error("invalid proof value: {0}")]
    InvalidProofValue(Bytes),
}

// required for IbcClient trait
impl From<Error> for IbcClientError<StarknetLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
