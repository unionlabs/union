use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha3::Sha3_256;

use crate::{hash_value::HashValue, transaction_info::TransactionInfo};

const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;

/// `TransactionInfo` and a `TransactionAccumulatorProof` connecting it to the ledger root.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionInfoWithProof {
    /// The accumulator proof from ledger info root to leaf that authenticates the hash of the
    /// `TransactionInfo` object.
    pub ledger_info_to_transaction_info_proof: TransactionAccumulatorProof,

    /// The `TransactionInfo` object at the leaf of the accumulator.
    pub transaction_info: TransactionInfo,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionAccumulatorProof {
    siblings: Vec<HashValue>,
}

#[derive(Debug)]
pub enum Error {
    RootHashMismatch,
}

impl TransactionAccumulatorProof {
    /// Verifies an element whose hash is `element_hash` and version is `element_version` exists in
    /// the accumulator whose root hash is `expected_root_hash` using the provided proof.
    pub fn verify(
        &self,
        expected_root_hash: HashValue,
        element_hash: HashValue,
        element_index: u64,
    ) -> Result<(), Error> {
        if self.siblings.len() > MAX_ACCUMULATOR_PROOF_DEPTH {
            // "Accumulator proof has more than {} ({}) siblings.",
            panic!("1");
        }

        let actual_root_hash = self
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
            return Err(Error::RootHashMismatch);
            // "{}: Root hashes do not match. Actual root hash: {:x}. Expected root hash: {:x}.",
        }

        Ok(())
    }
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
