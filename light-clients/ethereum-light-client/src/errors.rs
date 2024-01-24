use cosmwasm_std::StdError;
use ethereum_verifier::{
    ValidateLightClientError, VerifyAccountStorageRootError, VerifyStorageAbsenceError,
    VerifyStorageProofError,
};
use thiserror::Error as ThisError;
use unionlabs::{
    bls::BlsPublicKey,
    hash::{H160, H256},
    ibc::{core::client::height::Height, lightclients::ethereum::header::Header},
    TryFromProtoBytesError, TryFromProtoErrorOf,
};

use crate::Config;

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("error while decoding proto ({reason})")]
    DecodeFromProto { reason: String },

    #[error("client state not found")]
    ClientStateNotFound,

    #[error("invalid proof format ({0})")]
    InvalidProofFormat(String),

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

    #[error("{0}")]
    ValidateLightClient(#[from] ValidateLightClientError),

    #[error("{0}")]
    VerifyAccountStorageRoot(#[from] VerifyAccountStorageRootError),

    #[error("{0}")]
    VerifyStorageAbsence(#[from] VerifyStorageAbsenceError),

    #[error("{0}")]
    VerifyStorageProof(#[from] VerifyStorageProofError),

    #[error("IBC path is empty")]
    EmptyIbcPath,

    #[error("invalid commitment key, expected ({expected}) but found ({found})")]
    InvalidCommitmentKey { expected: H256, found: H256 },

    #[error("client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("proof is empty")]
    EmptyProof,

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error("batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("custom query ({0})")]
    CustomQuery(CustomQueryError),

    #[error("storage root mismatch, expected `{expected}` but found `{found}`")]
    StorageRootMismatch { expected: H256, found: H256 },

    #[error("wasm client error ({0})")]
    Wasm(String),

    #[error("next sync committee can't be changed after being set")]
    NextSyncCommitteeCannotBeModified,

    #[error("the slot number that is saved previously to the consensus state cannot be changed")]
    SlotCannotBeModified,

    #[error("the proof path {0} is not unknown")]
    UnknownIbcPath(String),

    #[error("the given contract address ({given}) doesn't match the stored value ({expected})")]
    IbcContractAddressMismatch { given: H160, expected: H160 },
}

impl From<TryFromProtoBytesError<TryFromProtoErrorOf<Header<Config>>>> for Error {
    fn from(value: TryFromProtoBytesError<TryFromProtoErrorOf<Header<Config>>>) -> Self {
        Self::DecodeFromProto {
            reason: format!("{:?}", value),
        }
    }
}

impl From<ics008_wasm_client::storage_utils::Error> for Error {
    fn from(error: ics008_wasm_client::storage_utils::Error) -> Self {
        match error {
            ics008_wasm_client::storage_utils::Error::ClientStateNotFound => {
                Error::ClientStateNotFound
            }
            ics008_wasm_client::storage_utils::Error::ClientStateDecode => Error::DecodeFromProto {
                reason: error.to_string(),
            },
            ics008_wasm_client::storage_utils::Error::ConsensusStateDecode => {
                Error::DecodeFromProto {
                    reason: error.to_string(),
                }
            }
        }
    }
}

#[derive(ThisError, Debug, PartialEq)]
pub enum CustomQueryError {
    #[error("error while running `fast_aggregate_verify` query ({0})")]
    FastAggregateVerify(String),
    #[error("error while running `aggregate_public_keys` query ({0})")]
    AggregatePublicKeys(String),
    #[error("invalid public key is returned from `aggregate_public_key`")]
    InvalidAggregatePublicKey,
}

impl From<CustomQueryError> for Error {
    fn from(value: CustomQueryError) -> Self {
        Error::CustomQuery(value)
    }
}
