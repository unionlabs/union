use milagro_bls::AmclError;
use ssz_rs::MerkleizationError;

#[derive(Debug)]
pub enum Error {
    InvalidMerkleBranch,
    Merkleization(MerkleizationError),
    InvalidChainVersion,
    Crypto,
    ExpectedCurrentSyncCommittee,
    EmptyAggregate,
    ExpectedNextSyncCommittee,
    IrrelevantUpdate,
    InvalidSlots,
    InvalidSignature,
    InvalidSignaturePeriod,
    InvalidPublicKey,
    NextSyncCommitteeMismatch,
    InsufficientSyncCommitteeParticipents,
    Bls(AmclError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidMerkleBranch => write!(f, "Invalid merkle branch."),
            Error::Merkleization(e) => write!(f, "Merkleization error: {e}"),
            Error::InvalidChainVersion => write!(f, "Invalid chain conversion."),
            Error::Crypto => write!(f, "Crypto error."),
            Error::ExpectedCurrentSyncCommittee => write!(f, "Expected current sync committee."),
            Error::ExpectedNextSyncCommittee => write!(f, "Expected next sync committee"),
            Error::IrrelevantUpdate => write!(f, "Irrelevant update."),
            Error::InvalidSlots => write!(f, "Invalid slots."),
            Error::InvalidSignaturePeriod => write!(
                f,
                "Signature period must be equal to `store_period` or `store_period + 1`"
            ),
            Error::NextSyncCommitteeMismatch => write!(
                f,
                "Next sync committee does not match with the one in the current state."
            ),
            Error::InsufficientSyncCommitteeParticipents => {
                write!(f, "Insufficient number of sync committee participants.")
            }
            Error::Bls(e) => write!(f, "Bls error: {e:?}"),
            Error::EmptyAggregate => write!(f, "Item list to be aggregated is empty."),
            Error::InvalidSignature => write!(f, "Signature is not valid."),
            Error::InvalidPublicKey => write!(f, "Invalid public key."),
        }
    }
}

impl From<MerkleizationError> for Error {
    fn from(e: MerkleizationError) -> Self {
        Error::Merkleization(e)
    }
}
