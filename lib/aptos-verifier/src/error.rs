use unionlabs::aptos::hash_value::HashValue;

use crate::MAX_ACCUMULATOR_PROOF_DEPTH;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("root hash mismatch, expected ({expected}) given ({given})")]
    RootHashMismatch {
        expected: HashValue,
        given: HashValue,
    },
    #[error("accumulator proof hash has more than maximum ({MAX_ACCUMULATOR_PROOF_DEPTH}) siblings ({0})")]
    MaxSiblingsExceeded(usize),
}
