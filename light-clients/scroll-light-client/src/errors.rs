use cosmwasm_std::StdError;
use ethereum_verifier::{
    ValidateLightClientError, VerifyAccountStorageRootError, VerifyStorageAbsenceError,
    VerifyStorageProofError,
};
use scroll_codec::{
    batch_header::BatchHeaderDecodeError,
    chunk::{ChunkV0DecodeError, ChunkV1DecodeError},
};
use thiserror::Error as ThisError;
use unionlabs::{
    hash::{H160, H256},
    ibc::core::client::height::Height,
};

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("error while decoding proto ({reason})")]
    DecodeFromProto { reason: String },

    #[error("client state not found")]
    ClientStateNotFound,

    #[error("invalid proof format ({0})")]
    InvalidProofFormat(String),

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

    #[error("proof is empty")]
    EmptyProof,

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error("batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("expected value ({expected}) and stored value ({stored}) don't match")]
    StoredValueMismatch { expected: H256, stored: H256 },

    #[error("storage root mismatch, expected `{expected}` but found `{found}`")]
    StorageRootMismatch { expected: H256, found: H256 },

    #[error("wasm client error ({0})")]
    Wasm(String),

    #[error("the proof path {0} is not unknown")]
    UnknownIbcPath(String),

    #[error("the given contract address ({given}) doesn't match the stored value ({expected})")]
    IbcContractAddressMismatch { given: H160, expected: H160 },

    #[error("failed to verify scroll header: {0}")]
    Verifier(#[from] scroll_verifier::Error),

    #[error("the operation has not been implemented yet")]
    Unimplemented,

    #[error("error while calling custom query: {0}")]
    CustomQuery(#[from] unionlabs::cosmwasm::wasm::union::custom_query::Error),

    #[error("error decoding commit batch calldata")]
    CommitBatchDecode(#[from] ethers_core::abi::AbiError),

    #[error("empty batch")]
    EmptyBatch,

    #[error("error decoding v0 chunk")]
    ChunkV0Decode(#[from] ChunkV0DecodeError),

    #[error("error decoding v1 chunk")]
    ChunkV1Decode(#[from] ChunkV1DecodeError),

    #[error("error decoding batch header")]
    BatchHeaderDecode(#[from] BatchHeaderDecodeError),
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
