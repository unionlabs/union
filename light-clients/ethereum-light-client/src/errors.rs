use cosmwasm_std::StdError;
use ibc_types::ibc::core::client::height::Height;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Client state not found")]
    ClientStateNotFound,

    #[error("Invalid sync committee")]
    InvalidSyncCommittee,

    #[error("Trusted sync committee's public key doesn't match the calculated public key. Calculated: {0}, Expected: {1}")]
    TrustedSyncCommitteePubkeyMismatch(String, String),

    #[error("Stored sync committee's public key doesn't match the trusted public key. Stored: {0}, Trusted: {1}")]
    StoredSyncCommitteePubkeyMismatch(String, String),

    #[error("No next sync committee")]
    NoNextSyncCommittee,

    #[error("Consensus state not found for {0:?}")]
    ConsensusStateNotFound(Height),

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("Invalid path {0}")]
    InvalidPath(String),

    #[error("Invalid commitment key. Expected {0}, got {1}.")]
    InvalidCommitmentKey(String, String),

    #[error("Client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("Proof is empty")]
    EmptyProof,

    #[error("Batching proofs are not supported")]
    BatchingProofsNotSupported,

    #[error("Expected value: '{0}' and stored value '{1}' doesn't match")]
    ExpectedAndStoredValueMismatch(String, String),

    #[error("Custom query: {0}")]
    CustomQuery(String),
}

impl Error {
    pub fn decode<S: ToString>(s: S) -> Error {
        Error::Decode(s.to_string())
    }

    pub fn invalid_path<S: ToString>(s: S) -> Error {
        Error::InvalidPath(s.to_string())
    }

    pub fn trusted_sync_committee_pubkey_mismatch<B1: AsRef<[u8]>, B2: AsRef<[u8]>>(
        trusted: B1,
        calculated: B2,
    ) -> Error {
        Error::TrustedSyncCommitteePubkeyMismatch(hex::encode(trusted), hex::encode(calculated))
    }

    pub fn stored_sync_committee_pubkey_mismatch<B1: AsRef<[u8]>, B2: AsRef<[u8]>>(
        stored: B1,
        trusted: B2,
    ) -> Error {
        Error::StoredSyncCommitteePubkeyMismatch(hex::encode(stored), hex::encode(trusted))
    }

    pub fn invalid_commitment_key<B1: AsRef<[u8]>, B2: AsRef<[u8]>>(
        expected: B1,
        got: B2,
    ) -> Error {
        Error::InvalidCommitmentKey(hex::encode(expected), hex::encode(got))
    }

    pub fn stored_value_mismatch<B1: AsRef<[u8]>, B2: AsRef<[u8]>>(expected: B1, got: B2) -> Error {
        Error::ExpectedAndStoredValueMismatch(hex::encode(expected), hex::encode(got))
    }

    pub fn custom_query<S: ToString>(s: S) -> Error {
        Error::CustomQuery(s.to_string())
    }
}
