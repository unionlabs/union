use trie_db::TrieError;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(
        "proof is invalid due to value mismatch, expected: {expected}, actual: {actual}",
        expected = serde_utils::to_hex(expected),
        actual = serde_utils::to_hex(actual)
    )]
    ValueMismatch { expected: Vec<u8>, actual: Vec<u8> },
    #[error("proof is invalid due to missing value: {v}", v = serde_utils::to_hex(value))]
    ValueMissing { value: Vec<u8> },
    #[error("trie error ({0:?})")]
    Trie(Box<TrieError<primitive_types::H256, rlp::DecoderError>>),
    #[error("rlp decoding failed: {0:?}")]
    RlpDecode(#[from] rlp::DecoderError),
}

// NOTE: Implemented here instead of via #[from] since Box<TrieError<primitive_types::H256, rlp::DecoderError>> doesn't implement core::error::Error
impl From<Box<TrieError<primitive_types::H256, rlp::DecoderError>>> for Error {
    fn from(e: Box<TrieError<primitive_types::H256, rlp::DecoderError>>) -> Self {
        Error::Trie(e)
    }
}
