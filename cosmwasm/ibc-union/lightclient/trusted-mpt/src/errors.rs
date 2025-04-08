use cosmwasm_std::Addr;
use ibc_union_light_client::IbcClientError;
use unionlabs::primitives::{H256, U256};

use crate::client::MptTrustedLightClient;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("misbehaviour logic is not needed in a trusted setup")]
    NoMisbehaviourInTrustedClient,

    #[error("unauthorized call")]
    Unauthorized,

    #[error("invalid contract address proof")]
    InvalidContractAddressProof(#[source] evm_storage_verifier::error::Error),

    #[error("invalid commitment key, expected ({expected:#x}) but found ({found:#x})")]
    InvalidCommitmentKey { expected: U256, found: U256 },

    #[error("verify storage proof error")]
    VerifyStorageProof(#[source] evm_storage_verifier::error::Error),

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("commitment key must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentKeyLength(Vec<u8>),

    #[error("commitment value must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentValueLength(Vec<u8>),

    #[error("verify storage absence error")]
    VerifyStorageAbsence(#[source] evm_storage_verifier::error::Error),

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,
}

impl From<Error> for IbcClientError<MptTrustedLightClient> {
    fn from(value: Error) -> Self {
        IbcClientError::ClientSpecific(value)
    }
}
