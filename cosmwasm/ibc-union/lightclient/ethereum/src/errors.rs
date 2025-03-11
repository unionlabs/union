use beacon_api_types::slot::Slot;
use cosmwasm_std::{StdError, VerificationError};
use ibc_union_light_client::IbcClientError;
use unionlabs::primitives::{H256, U256};

use crate::client::EthereumLightClient;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("validate light client error")]
    ValidateLightClient(#[source] ethereum_sync_protocol::error::Error),

    #[error("verify account storage root error")]
    VerifyAccountStorageRoot(#[source] evm_storage_verifier::error::Error),

    #[error("verify storage absence error")]
    VerifyStorageAbsence(#[source] evm_storage_verifier::error::Error),

    #[error("verify storage proof error")]
    VerifyStorageProof(#[source] evm_storage_verifier::error::Error),

    #[error("commitment key must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentKeyLength(Vec<u8>),

    #[error("commitment value must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentValueLength(Vec<u8>),

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error("not enough signatures")]
    NotEnoughSignatures,

    #[error("integer arithmetic overflow")]
    IntegerOverflow,

    #[error("misbehaviour can only exist if there exists two conflicting headers, the provided headers are not at the same height ({0} != {1})")]
    MisbehaviourCannotExist(Slot, Slot),

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("invalid commitment key, expected ({expected:#x}) but found ({found:#x})")]
    InvalidCommitmentKey { expected: U256, found: U256 },

    // REVIEW: Unused?
    #[error(
        "client state's latest slot ({client_state_latest_slot}) \
        expected to be equal to consensus state's slot ({consensus_state_slot})"
    )]
    InvalidInitialState {
        client_state_latest_slot: u64,
        consensus_state_slot: u64,
    },

    #[error("the misbehaviour headers must be different")]
    IdenticalMisbehaviourHeaders,

    #[error("aggregate pubkey mismatch")]
    AggregatePubkeyMismatch,

    #[error(transparent)]
    VerificationError(#[from] VerificationError),

    #[error("the initial sync committee must be provided during client creation")]
    NoInitialSyncCommittee,
}

impl From<Error> for StdError {
    fn from(value: Error) -> Self {
        StdError::generic_err(value.to_string())
    }
}

impl From<Error> for IbcClientError<EthereumLightClient> {
    fn from(value: Error) -> Self {
        Self::ClientSpecific(value)
    }
}
