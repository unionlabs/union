use ibc_types::ethereum::H256;
use milagro_bls::AmclError;
use trie_db::TrieError;

#[derive(Debug, PartialEq)]
pub struct InvalidMerkleBranch {
    pub leaf: H256,
    pub branch: Vec<H256>,
    pub depth: usize,
    pub index: u64,
    pub root: H256,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidMerkleBranch(InvalidMerkleBranch),
    InvalidChainVersion(u64),
    Crypto,
    ExpectedCurrentSyncCommittee,
    ExpectedNextSyncCommittee,
    IrrelevantUpdate,
    InvalidSlots {
        current_slot: u64,
        updates_signature_slot: u64,
        updates_attested_slot: u64,
        updates_finalized_slot: u64,
    },
    InvalidSignature,
    InvalidNextSyncCommitteePeriod {
        update_period: u64,
        stored_period: u64,
    },
    InvalidCurrentSyncCommitteePeriod {
        update_period: u64,
        stored_period: u64,
    },
    NextSyncCommitteeMismatch {
        stored_next_sync_committee: Vec<u8>,
        updates_next_sync_committee: Vec<u8>,
    },
    InsufficientSyncCommitteeParticipants {
        expected_count: usize,
        actual_count: usize,
    },
    Bls(AmclError),
    ProofMismatch {
        expected: Vec<u8>,
        got: Vec<u8>,
    },
    ProofNotFound,
    Trie(Box<TrieError<primitive_types::H256, rlp::DecoderError>>),
    RlpDecode,
    External(String),
}

impl Error {
    pub fn invalid_slots(
        current_slot: u64,
        updates_signature_slot: u64,
        updates_attested_slot: u64,
        updates_finalized_slot: u64,
    ) -> Error {
        Error::InvalidSlots {
            current_slot,
            updates_signature_slot,
            updates_attested_slot,
            updates_finalized_slot,
        }
    }

    pub fn invalid_next_sync_committee_period(update_period: u64, stored_period: u64) -> Error {
        Error::InvalidNextSyncCommitteePeriod {
            update_period,
            stored_period,
        }
    }

    pub fn invalid_current_sync_committee_period(update_period: u64, stored_period: u64) -> Error {
        Error::InvalidCurrentSyncCommitteePeriod {
            update_period,
            stored_period,
        }
    }

    pub fn next_sync_committee_mismatch<B1: Into<Vec<u8>>, B2: Into<Vec<u8>>>(
        stored_next_sync_committee: B1,
        updates_next_sync_committee: B2,
    ) -> Error {
        Error::NextSyncCommitteeMismatch {
            stored_next_sync_committee: stored_next_sync_committee.into(),
            updates_next_sync_committee: updates_next_sync_committee.into(),
        }
    }

    pub fn insufficient_sync_committee_participants(
        expected_count: usize,
        actual_count: usize,
    ) -> Error {
        Error::InsufficientSyncCommitteeParticipants {
            expected_count,
            actual_count,
        }
    }

    pub fn proof_mismatch<B1: Into<Vec<u8>>, B2: Into<Vec<u8>>>(expected: B1, got: B2) -> Error {
        Error::ProofMismatch {
            expected: expected.into(),
            got: got.into(),
        }
    }

    pub fn external<S: ToString>(s: S) -> Error {
        Error::External(s.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidMerkleBranch(err) => write!(f, "Invalid merkle branch: {err:#?}"),
            Error::InvalidChainVersion(epoch) => {
                write!(f, "Invalid chain conversion. Epoch: {epoch}")
            }
            Error::Crypto => write!(f, "Crypto error."),
            Error::ExpectedCurrentSyncCommittee => write!(f, "Expected current sync committee."),
            Error::ExpectedNextSyncCommittee => write!(f, "Expected next sync committee"),
            Error::IrrelevantUpdate => write!(f, "Irrelevant update."),
            Error::InvalidSlots {
                current_slot,
                updates_signature_slot,
                updates_attested_slot,
                updates_finalized_slot,
            } => {
                write!(f, "Invalid update: `current_slot ({current_slot})>= update.signature_slot ({updates_signature_slot}) > update_attested_slot ({updates_attested_slot}) >= update_finalized_slot ({updates_finalized_slot})` is false.")
            }
            Error::InvalidNextSyncCommitteePeriod {
                update_period,
                stored_period,
            } => {
                write!(f, "Signature period must be equal to `store_period` or `store_period + 1`. Update period: {update_period}. Stored period: {stored_period}.")
            }
            Error::InvalidCurrentSyncCommitteePeriod {
                update_period,
                stored_period,
            } => {
                write!(f,"Signature period must be equal to `store_period`. Update period: {update_period}. Stored period: {stored_period}.")
            }
            Error::NextSyncCommitteeMismatch {
                stored_next_sync_committee,
                updates_next_sync_committee,
            } => {
                write!(
                    f,
                    "Next sync committee does not match with the one in the current state. Stored: {}. Update's: {}.", hex::encode(stored_next_sync_committee), hex::encode(updates_next_sync_committee)
                )
            }
            Error::InsufficientSyncCommitteeParticipants {
                expected_count,
                actual_count,
            } => {
                write!(f, "Insufficient number of sync committee participants. Expected at least: {expected_count}, participated: {actual_count}.")
            }
            Error::Bls(e) => write!(f, "Bls error: {e:?}"),
            Error::InvalidSignature => write!(f, "Signature is not valid."),
            Error::ProofMismatch { expected, got } => write!(
                f,
                "Proof is invalid. Value mismatch. Expected: {}, got: {}",
                hex::encode(expected),
                hex::encode(got),
            ),
            Error::Trie(e) => write!(f, "Trie error: {e:?}"),
            Error::RlpDecode => write!(f, "Rlp decoding failed."),
            Error::External(e) => write!(f, "Custom query error: {}", e),
            Error::ProofNotFound => write!(f, "Value not found in the merkle proof."),
        }
    }
}

impl From<AmclError> for Error {
    fn from(e: AmclError) -> Self {
        Error::Bls(e)
    }
}

impl From<Box<TrieError<primitive_types::H256, rlp::DecoderError>>> for Error {
    fn from(e: Box<TrieError<primitive_types::H256, rlp::DecoderError>>) -> Self {
        Error::Trie(e)
    }
}
