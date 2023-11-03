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

    #[error("verification error: {error} ({context})")]
    Verification { context: String, error: String },

    #[error("invalid path ({0})")]
    InvalidPath(String),

    #[error("invalid commitment key, expected ({0}) but got ({1})")]
    InvalidCommitmentKey(String, String),

    #[error("client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("proof is empty")]
    EmptyProof,

    #[error("counterparty storage not nil")]
    CounterpartyStorageNotNil,

    #[error("batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("expected value ({0}) and stored value ({1}) doesn't match")]
    ExpectedAndStoredValueMismatch(String, String),

    #[error("custom query ({0})")]
    CustomQuery(String),

    #[error("storage root mismatch, expected ({0}) but got ({1})")]
    StorageRootMismatch(String, String),

    #[error("wasm client error ({0})")]
    Wasm(String),

    #[error("next sync committee can't be changed after being set")]
    NextSyncCommitteeCannotBeModified,

    #[error("the slot number that is saved previously to the consensus state cannot be changed")]
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
            ics008_wasm_client::Error::UnexpectedCallDataFromHostModule(e) => Error::Wasm(e),
            ics008_wasm_client::Error::ClientStateNotFound => Error::ClientStateNotFound,
        }
    }
}
