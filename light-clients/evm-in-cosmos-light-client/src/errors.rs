use ethereum_light_client::errors::CanonicalizeStoredValueError;
use ics008_wasm_client::IbcClientError;
use unionlabs::{
    encoding::{DecodeErrorOf, Proto},
    ibc::{
        core::client::height::Height,
        lightclients::{ethereum::storage_proof::StorageProof, evm_in_cosmos},
    },
};

use crate::client::EvmInCosmosLightClient;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("unimplemented feature")]
    Unimplemented,

    #[error("unable to decode storage proof")]
    StorageProofDecode(#[source] DecodeErrorOf<Proto, StorageProof>),

    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, evm_in_cosmos::client_state::ClientState>),

    #[error(transparent)]
    CanonicalizeStoredValue(#[from] CanonicalizeStoredValueError),

    #[error("custom query error")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),

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
