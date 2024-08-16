use std::fmt;

use hex::FromHex;
use hex_literal::hex;
use serde::{de, ser, Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::hash_value::HashValue;

// "SPARSE_MERKLE_PLACEHOLDER_HASH"
pub const SPARSE_MERKLE_PLACEHOLDER_HASH: HashValue = HashValue(hex!(
    "00005350415253455F4D45524B4C455F504C414345484F4C4445525F48415348"
));

const LENGTH: usize = 32;
const LENGTH_IN_BITS: usize = 32 * 8;

/// A proof that can be used to authenticate an element in a Sparse Merkle Tree given trusted root
/// hash. For example, `TransactionInfoToAccountProof` can be constructed on top of this structure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SparseMerkleProof {
    /// This proof can be used to authenticate whether a given leaf exists in the tree or not.
    ///     - If this is `Some(leaf_node)`
    ///         - If `leaf_node.key` equals requested key, this is an inclusion proof and
    ///           `leaf_node.value_hash` equals the hash of the corresponding account blob.
    ///         - Otherwise this is a non-inclusion proof. `leaf_node.key` is the only key
    ///           that exists in the subtree and `leaf_node.value_hash` equals the hash of the
    ///           corresponding account blob.
    ///     - If this is `None`, this is also a non-inclusion proof which indicates the subtree is
    ///       empty.
    leaf: Option<SparseMerkleLeafNode>,

    /// All siblings in this proof, including the default ones. Siblings are ordered from the root
    /// level to the bottom level.
    siblings: Vec<HashValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SparseMerkleLeafNode {
    key: HashValue,
    value_hash: HashValue,
}

pub struct SparseMerkleInternalNode {
    left_child: HashValue,
    right_child: HashValue,
}

impl SparseMerkleInternalNode {
    pub fn new(left_child: HashValue, right_child: HashValue) -> Self {
        Self {
            left_child,
            right_child,
        }
    }

    pub fn hash(&self) -> HashValue {
        let mut state = Sha3_256::new();
        state.update(
            Sha3_256::new()
                .chain_update("APTOS::SparseMerkleInternal")
                .finalize(),
        );
        state.update(self.left_child.as_ref());
        state.update(self.right_child.as_ref());
        HashValue(state.finalize().into())
    }
}

impl SparseMerkleLeafNode {
    pub fn new(key: HashValue, value_hash: HashValue) -> Self {
        SparseMerkleLeafNode { key, value_hash }
    }

    pub fn hash(&self) -> HashValue {
        let mut state = Sha3_256::new();
        state.update(
            Sha3_256::new()
                .chain_update("APTOS::SparseMerkleLeafNode")
                .finalize(),
        );
        state.update(self.key.as_ref());
        state.update(self.value_hash.as_ref());
        HashValue(state.finalize().into())
    }
}

impl SparseMerkleProof {
    pub fn verify_by_hash(
        &self,
        expected_root_hash: HashValue,
        element_key: HashValue,
        element_hash: Option<HashValue>,
    ) {
        self.verify_by_hash_partial(expected_root_hash, element_key, element_hash, 0)
    }

    pub fn verify_by_hash_partial(
        &self,
        expected_root_hash: HashValue,
        element_key: HashValue,
        element_hash: Option<HashValue>,
        root_depth: usize,
    ) {
        if self.siblings.len() + root_depth > HashValue::LENGTH_IN_BITS {
            // "Sparse Merkle Tree proof has more than {} ({} + {}) siblings.",
            return;
        }

        match (element_hash, self.leaf) {
            (Some(hash), Some(leaf)) => {
                // This is an inclusion proof, so the key and value hash provided in the proof
                // should match element_key and element_value_hash. `siblings` should prove the
                // route from the leaf node to the root.
                if element_key != leaf.key {
                    panic!("Keys do not match.");
                    //     Key in proof: {:x}. Expected key: {:x}. \
                    // Element hash: {:x}. Value hash in proof {:x}",
                }
                if hash != leaf.value_hash {
                    panic!("Value hashes do not match");
                    // for key {:x}. Value hash in proof: {:x}. \
                    //  Expected value hash: {:x}. ",
                }
            }
            (Some(_), None) => {
                panic!("Expected inclusion proof");
                // , value hash: {:x}. Found non-inclusion proof.",
            }
            (None, Some(leaf)) => {
                // This is a non-inclusion proof. The proof intends to show that if a leaf node
                // representing `element_key` is inserted, it will break a currently existing leaf
                // node represented by `proof_key` into a branch. `siblings` should prove the
                // route from that leaf node to the root.
                if element_key == leaf.key {
                    panic!("Expected non-inclusion proof");
                    // "Expected non-inclusion proof, but key exists in proof. \
                    //  Key: {:x}. Key in proof: {:x}.",
                }
                if element_key.common_prefix_bits_len(leaf.key) < root_depth + self.siblings.len() {
                    panic!("Keys would have");
                    // "Key would not have ended up in the subtree where the provided key in proof \
                    //  is the only existing key, if it existed. So this is not a valid \
                    //  non-inclusion proof. Key: {:x}. Key in proof: {:x}.",
                }
            }
            (None, None) => {
                // This is a non-inclusion proof. The proof intends to show that if a leaf node
                // representing `element_key` is inserted, it will show up at a currently empty
                // position. `sibling` should prove the route from this empty position to the root.
            }
        }

        let current_hash = self
            .leaf
            .map_or(SPARSE_MERKLE_PLACEHOLDER_HASH, |leaf| leaf.hash());
        let actual_root_hash = self
            .siblings
            .iter()
            .rev()
            .zip(
                element_key
                    .iter_bits()
                    .rev()
                    .skip(HashValue::LENGTH_IN_BITS - self.siblings.len() - root_depth),
            )
            .fold(current_hash, |hash, (sibling_hash, bit)| {
                if bit {
                    SparseMerkleInternalNode::new(*sibling_hash, hash).hash()
                } else {
                    SparseMerkleInternalNode::new(hash, *sibling_hash).hash()
                }
            });
        if actual_root_hash != expected_root_hash {
            panic!("Root hashes do not match.")
        }

        // Ok(())
    }
}
