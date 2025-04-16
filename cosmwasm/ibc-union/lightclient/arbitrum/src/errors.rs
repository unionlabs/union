use ethereum_light_client::client::EthereumLightClient;
use ibc_union_light_client::IbcClientError;
use unionlabs::{
    ibc::core::client::height::Height,
    primitives::{H256, U256},
};

use crate::client::ArbitrumLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Evm(#[from] ethereum_light_client::errors::Error),

    // REVIEW: Move this variant to IbcClientError?
    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("the l2 height {0} is too large (> u64::MAX)")]
    L2HeightTooLarge(U256),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("proof is empty")]
    EmptyProof,

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("failed to verify arbitrum header: {0}")]
    HeaderVerify(#[from] arbitrum_verifier::Error),

    #[error("the operation has not been implemented yet")]
    Unimplemented,

    #[error(transparent)]
    EvmIbcClient(#[from] IbcClientError<EthereumLightClient>),
}

impl From<Error> for IbcClientError<ArbitrumLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
