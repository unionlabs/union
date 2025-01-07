use cometbls_light_client::client::CometblsLightClient;
use cosmwasm_std::StdError;
use ibc_union_light_client::IbcClientError;
use unionlabs::{hash::H256, ibc::core::client::height::Height, uint::U256};

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

    #[error("error while querying l1 state: {0}")]
    L1Error(#[from] IbcClientError<CometblsLightClient>),

    #[error("commitment key must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentKeyLength(Vec<u8>),

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("verify storage proof error")]
    VerifyStorageProof(#[source] evm_storage_verifier::error::Error),

    #[error("invalid commitment key, expected ({expected:#x}) but found ({found:#x})")]
    InvalidCommitmentKey { expected: U256, found: U256 },

    #[error("commitment value must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentValueLength(Vec<u8>),

    #[error("verify storage absence error")]
    VerifyStorageAbsence(#[source] evm_storage_verifier::error::Error),

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,
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
