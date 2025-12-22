use ethereum_light_client::client::EthereumLightClient;
use ibc_union_light_client::IbcClientError;
use unionlabs::primitives::U256;

use crate::client::ArbitrumLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    EthereumLightClient(#[from] IbcClientError<EthereumLightClient>),

    #[error("the l2 height {0} is too large (> u64::MAX)")]
    L2HeightTooLarge(U256),

    #[error("failed to verify arbitrum header (v1)")]
    HeaderVerifyV1(#[from] arbitrum_verifier::v1::Error),

    #[error("failed to verify arbitrum header (v2)")]
    HeaderVerifyV2(#[from] arbitrum_verifier::v2::Error),

    #[error("the operation has not been implemented yet")]
    Unimplemented,

    #[error("invalid header, must be v1")]
    HeaderMustBeV1,

    #[error("invalid header, must be v2")]
    HeaderMustBeV2,
}

impl From<Error> for IbcClientError<ArbitrumLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
