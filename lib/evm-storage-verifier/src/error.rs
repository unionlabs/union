use trie_db::TrieError;
use unionlabs::primitives::{Bytes, H256};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("proof is invalid due to value mismatch, expected: {expected}, actual: {actual}")]
    ValueMismatch { expected: Bytes, actual: Bytes },
    #[error("proof is invalid due to missing value: {value}")]
    ValueMissing { value: Bytes },
    #[error("trie error ({0:?})")]
    Trie(Box<TrieError<H256, rlp::DecoderError>>),
    #[error("rlp decoding failed: {0:?}")]
    RlpDecode(#[from] rlp::DecoderError),
}

// NOTE: Implemented here instead of via #[from] since Box<TrieError<H256, rlp::DecoderError>> doesn't implement core::error::Error
impl From<Box<TrieError<H256, rlp::DecoderError>>> for Error {
    fn from(e: Box<TrieError<H256, rlp::DecoderError>>) -> Self {
        Error::Trie(e)
    }
}
