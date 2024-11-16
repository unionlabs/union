use cosmwasm_std::StdError;
use ethereum_light_client_types::{client_state, consensus_state, StorageProof};
use union_ibc_light_client::IbcClientError;
use unionlabs::{
    bls::BlsPublicKey,
    encoding::{DecodeErrorOf, Proto},
    hash::H256,
    ibc::core::client::height::Height,
    uint::U256,
};

use crate::client::EthereumLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("unimplemented feature")]
    Unimplemented,

    #[error("unable to decode storage proof")]
    StorageProofDecode(#[source] DecodeErrorOf<Proto, StorageProof>),

    #[error("client state not found")]
    ClientStateNotFound,

    #[error("unable to decode client state")]
    ClientStateDecode(#[source] DecodeErrorOf<Proto, client_state::ClientState>),
    #[error("unable to decode consensus state")]
    ConsensusStateDecode(#[source] DecodeErrorOf<Proto, consensus_state::ConsensusState>),

    #[error("custom query error")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),

    #[error(
        "given trusted sync committee doesn't match the given aggregate public \
        key ({given_aggregate}) or the stored one ({stored_aggregate})"
    )]
    TrustedSyncCommitteeMismatch {
        stored_aggregate: BlsPublicKey,
        given_aggregate: BlsPublicKey,
    },

    #[error("active sync committee is `next` but there is no next sync committee in the consensus state")]
    NoNextSyncCommittee,

    #[error("consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("validate light client error")]
    ValidateLightClient(#[source] ethereum_sync_protocol::error::Error),

    #[error("verify account storage root error")]
    VerifyAccountStorageRoot(#[source] evm_storage_verifier::error::Error),

    #[error("verify storage absence error")]
    VerifyStorageAbsence(#[source] evm_storage_verifier::error::Error),

    #[error("verify storage proof error")]
    VerifyStorageProof(#[source] evm_storage_verifier::error::Error),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("commitment key must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentKeyLength(Vec<u8>),

    #[error("commitment value must be 32 bytes but we got: {0:?}")]
    InvalidCommitmentValueLength(Vec<u8>),

    #[error("client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("proof is empty")]
    EmptyProof,

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error("not enough signatures")]
    NotEnoughSignatures,

    #[error("integer arithmetic overflow")]
    IntegerOverflow,

    #[error("forbidden fields have been changed during state migration")]
    MigrateFieldsChanged,

    #[error("substitute client is frozen")]
    SubstituteClientFrozen,

    #[error("misbehaviour can only exist if there exists two conflicting headers, the provided headers are not at the same height ({0} != {1})")]
    MisbehaviourCannotExist(u64, u64),

    #[error(transparent)]
    StdError(#[from] StdError),

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("invalid commitment key, expected ({expected:#x}) but found ({found:#x})")]
    InvalidCommitmentKey { expected: U256, found: U256 },

    #[error(
        "client state's latest slot ({client_state_latest_slot}) \
        expected to be equal to consensus state's slot ({consensus_state_slot})"
    )]
    InvalidInitialState {
        client_state_latest_slot: u64,
        consensus_state_slot: u64,
    },
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
