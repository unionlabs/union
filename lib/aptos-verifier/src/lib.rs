pub mod error;

use error::Error;
use sha3::{Digest, Sha3_256};
use unionlabs::aptos::{hash_value::HashValue, transaction_proof::TransactionAccumulatorProof};

pub(crate) const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;

/// Verifies an element whose hash is `element_hash` and version is `element_version` exists in
/// the accumulator whose root hash is `expected_root_hash` using the provided proof.
pub fn verify_tx_accumulator_proof(
    proof: &TransactionAccumulatorProof,
    expected_root_hash: HashValue,
    element_hash: HashValue,
    element_index: u64,
) -> Result<(), Error> {
    if proof.siblings.len() > MAX_ACCUMULATOR_PROOF_DEPTH {
        return Err(Error::MaxSiblingsExceeded(proof.siblings.len()));
    }

    let actual_root_hash = proof
        .siblings
        .iter()
        .fold(
            (element_hash, element_index),
            // `index` denotes the index of the ancestor of the element at the current level.
            |(hash, index), sibling_hash| {
                (
                    if index % 2 == 0 {
                        // the current node is a left child.
                        hash_inner_node(hash, *sibling_hash)
                    } else {
                        // the current node is a right child.
                        hash_inner_node(*sibling_hash, hash)
                    },
                    // The index of the parent at its level.
                    index / 2,
                )
            },
        )
        .0;

    if actual_root_hash != expected_root_hash {
        return Err(Error::RootHashMismatch {
            expected: expected_root_hash,
            given: actual_root_hash,
        });
    }

    Ok(())
}

pub fn hash_inner_node(left_child: HashValue, right_child: HashValue) -> HashValue {
    let mut state = Sha3_256::new();
    state.update(
        Sha3_256::new()
            .chain_update("APTOS::TransactionAccumulator")
            .finalize(),
    );
    state.update(left_child.as_ref());
    state.update(right_child.as_ref());
    HashValue(state.finalize().into())
}
