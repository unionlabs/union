use milagro_bls::AmclError;
use trie_db::TrieError;
use unionlabs::ethereum::H256;

#[derive(Debug, PartialEq)]
pub struct InvalidMerkleBranch {
    pub leaf: H256,
    pub branch: Vec<H256>,
    pub depth: usize,
    pub index: u64,
    pub root: H256,
}

#[derive(Debug, PartialEq, derive_more::Display)]
pub enum Error {
    #[display(fmt = "invalid merkle branch ({:?})", "_0")]
    InvalidMerkleBranch(InvalidMerkleBranch),
    #[display(fmt = "invalid chain conversion")]
    InvalidChainVersion,
    #[display(fmt = "crypto error")]
    Crypto,
    #[display(
        fmt = "expected current sync committee to be provided since `update_period == current_period`"
    )]
    ExpectedCurrentSyncCommittee,
    #[display(
        fmt = "expected next sync committee to be provided since `update_period > current_period`"
    )]
    ExpectedNextSyncCommittee,
    #[display(
        fmt = "irrelevant update since the order of the slots in the update data, and stored data is not correct"
    )]
    IrrelevantUpdate,
    #[display(fmt = "the order of the slots in the update data, and stored data is not correct")]
    InvalidSlots,
    #[display(
        fmt = "signature period must be equal to `store_period` or `store_period + 1` \
                when the next sync committee is stored. Otherwise, it must be equal to `store_period`"
    )]
    InvalidSignaturePeriod,
    #[display(fmt = "signature is not valid")]
    InvalidSignature,
    #[display(fmt = "invalid public key")]
    InvalidPublicKey,
    #[display(fmt = "next sync committee does not match with the one in the current state")]
    NextSyncCommitteeMismatch,
    #[display(
        fmt = "insufficient number of sync committee participants, expected it to be at least ({}) but got ({})",
        "_0",
        "_1"
    )]
    InsufficientSyncCommitteeParticipants(usize, usize),
    #[display(fmt = "Bls error ({:?})", "_0")]
    Bls(AmclError),
    #[display(fmt = "proof is invalid due to value mismatch")]
    ValueMismatch,
    #[display(fmt = "trie error ({:?})", "_0")]
    Trie(Box<TrieError<primitive_types::H256, rlp::DecoderError>>),
    #[display(fmt = "rlp decoding failed ({})", "_0")]
    RlpDecode(String),
    #[display(fmt = "custom query error: ({})", "_0")]
    CustomError(String),
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
