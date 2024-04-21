use cosmwasm_std::StdError;
use ethereum_verifier::{
    ValidateLightClientError, VerifyAccountStorageRootError, VerifyStorageAbsenceError,
    VerifyStorageProofError,
};
use thiserror::Error as ThisError;
use unionlabs::{bls::BlsPublicKey, hash::H256, ibc::core::client::height::Height};

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unimplemented feature")]
    Unimplemented,

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

    #[error("expected value ({expected:?}) and stored value ({stored:?}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

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

    #[error("error while calling custom query: {0}")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),

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

    #[error("unknown checkpoint index {0}")]
    UnknownCheckpointIndex(u64),
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
