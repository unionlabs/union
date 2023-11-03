use cosmwasm_std::StdError;
use thiserror::Error as ThisError;
use unionlabs::{
    bls::BlsPublicKey,
    ibc::{core::client::height::Height, lightclients::ethereum::header::Header},
    TryFromProtoBytesError, TryFromProtoErrorOf,
};

use crate::Config;

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Error while decoding proto: {reason}")]
    DecodeFromProto { reason: String },

    #[error("Client state not found")]
    ClientStateNotFound,

    #[error("Invalid proof format: `{0}`")]
    InvalidProofFormat(String),

    #[error(
        "Given trusted sync committee doesn't match the given aggregate \
        public key, or the stored one. Stored aggregate: {stored_aggregate}, \
        given aggregate: {given_aggregate}"
    )]
    TrustedSyncCommitteeMismatch {
        stored_aggregate: BlsPublicKey,
        given_aggregate: BlsPublicKey,
    },

    #[error("Active sync committee is `next` but there is no next sync committee in the consensus state")]
    NoNextSyncCommittee,

    #[error("Consensus state not found at height {0}")]
    ConsensusStateNotFound(Height),

    #[error("Verification error: {error} ({context})")]
    Verification { context: String, error: String },

    #[error("Invalid path {0}")]
    InvalidPath(String),

    #[error("Invalid commitment key. Expected {0}, got {1}.")]
    InvalidCommitmentKey(String, String),

    #[error("Client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("Proof is empty")]
    EmptyProof,

    #[error("Counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error("Batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("Expected value: '{0}' and stored value '{1}' doesn't match")]
    ExpectedAndStoredValueMismatch(String, String),

    #[error("Custom query: {0}")]
    CustomQuery(String),

    #[error("Storage root mismatch. Expected {0}, got {1}")]
    StorageRootMismatch(String, String),

    #[error("Wasm client error: {0}")]
    Wasm(String),

    #[error("Next sync committee can't be changed after being set.")]
    NextSyncCommitteeCannotBeModified,

    #[error("The slot number that is saved previously to the consensus state cannot be changed.")]
    SlotCannotBeModified,
}

impl Error {
    pub fn invalid_commitment_key<B1: AsRef<[u8]>, B2: AsRef<[u8]>>(
        expected: B1,
        got: B2,
    ) -> Error {
        Error::InvalidCommitmentKey(hex::encode(expected), hex::encode(got))
    }

    pub fn stored_value_mismatch<B1: AsRef<[u8]>, B2: AsRef<[u8]>>(expected: B1, got: B2) -> Error {
        Error::ExpectedAndStoredValueMismatch(hex::encode(expected), hex::encode(got))
    }
}

impl From<TryFromProtoBytesError<TryFromProtoErrorOf<Header<Config>>>> for Error {
    fn from(value: TryFromProtoBytesError<TryFromProtoErrorOf<Header<Config>>>) -> Self {
        Self::DecodeFromProto {
            reason: format!("{:?}", value),
        }
    }
}

impl From<ics008_wasm_client::Error> for Error {
    fn from(error: ics008_wasm_client::Error) -> Self {
        match error {
            ics008_wasm_client::Error::Decode(e) => Error::DecodeFromProto { reason: e },
            // TODO(aeryz): what
            ics008_wasm_client::Error::UnexpectedCallDataFromHostModule(e) => Error::Wasm(e),
            ics008_wasm_client::Error::ClientStateNotFound => Error::ClientStateNotFound,
        }
    }
}
