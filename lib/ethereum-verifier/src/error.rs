use ssz_rs::MerkleizationError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid merkle branch.")]
    InvalidMerkleBranch,

    #[error("Merkleization error: {0}")]
    Merkleization(MerkleizationError),

    #[error("Invalid chain version")]
    InvalidChainVersion,

    #[error("Crypto error: {0}")]
    Crypto(ethereum_consensus::crypto::Error),

    #[error("State transition error: {0}")]
    StateTransition(ethereum_consensus::state_transition::Error),

    #[error("Expected current sync committee.")]
    ExpectedCurrentSyncCommittee,

    #[error("Expected next sync committee.")]
    ExpectedNextSyncCommittee,

    #[error("Irrelevant update.")]
    IrrelevantUpdate,

    #[error("Invalid slots.")]
    InvalidSlots,

    #[error("Signature period must be equal to `store_period` or `store_period + 1`")]
    InvalidSignaturePeriod,

    #[error("Next sync committee does not match with the one in the current state.")]
    NextSyncCommitteeMismatch,

    #[error("Insufficient number of sync committee participants.")]
    InsufficientSyncCommitteeParticipents,
}

impl From<MerkleizationError> for Error {
    fn from(e: MerkleizationError) -> Self {
        Error::Merkleization(e)
    }
}

impl From<ethereum_consensus::crypto::Error> for Error {
    fn from(e: ethereum_consensus::crypto::Error) -> Self {
        Error::Crypto(e)
    }
}

impl From<ethereum_consensus::state_transition::Error> for Error {
    fn from(e: ethereum_consensus::state_transition::Error) -> Self {
        Error::StateTransition(e)
    }
}
