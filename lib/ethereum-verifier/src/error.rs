use ibc_types::ethereum::H256;
use milagro_bls::AmclError;
use trie_db::TrieError;

#[derive(Debug)]
pub struct InvalidMerkleBranch {
    pub leaf: H256,
    pub branch: Vec<H256>,
    pub depth: usize,
    pub index: u64,
    pub root: H256,
}

#[derive(Debug)]
pub enum Error {
    InvalidMerkleBranch(InvalidMerkleBranch),
    InvalidChainVersion,
    Crypto,
    ExpectedCurrentSyncCommittee,
    ExpectedNextSyncCommittee,
    IrrelevantUpdate,
    InvalidSlots,
    InvalidSignature,
    InvalidSignaturePeriod,
    InvalidPublicKey,
    NextSyncCommitteeMismatch,
    InsufficientSyncCommitteeParticipants,
    Bls(AmclError),
    ValueMismatch,
    Trie(Box<TrieError<primitive_types::H256, rlp::DecoderError>>),
    RlpDecode,
    InvalidHash,
    CustomError(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidMerkleBranch(err) => write!(f, "Invalid merkle branch: {err:#?}"),
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
            Error::InsufficientSyncCommitteeParticipants => {
                write!(f, "Insufficient number of sync committee participants.")
            }
            Error::Bls(e) => write!(f, "Bls error: {e:?}"),
            Error::InvalidSignature => write!(f, "Signature is not valid."),
            Error::InvalidPublicKey => write!(f, "Invalid public key."),
            Error::ValueMismatch => write!(f, "Proof is invalid. Value mismatch."),
            Error::Trie(e) => write!(f, "Trie error: {e:?}"),
            Error::RlpDecode => write!(f, "Rlp decoding failed."),
            Error::InvalidHash => write!(f, "Invalid hash."),
            Error::CustomError(e) => write!(f, "Custom query error: {}", e),
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
