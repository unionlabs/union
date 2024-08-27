pub mod error;

pub use error::Error;
use error::StorageVerificationError;
use hex_literal::hex;
use sha3::{Digest, Sha3_256};
use unionlabs::{
    aptos::{
        hash_value::HashValue,
        sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        transaction_info::TransactionInfo,
        transaction_proof::TransactionInfoWithProof,
    },
    encoding::{DecodeAs, Proto},
};

pub(crate) const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
// "SPARSE_MERKLE_PLACEHOLDER_HASH"
pub(crate) const SPARSE_MERKLE_PLACEHOLDER_HASH: HashValue = HashValue(hex!(
    "00005350415253455F4D45524B4C455F504C414345484F4C4445525F48415348"
));

/// Verifies an element whose hash is `element_hash` and version is `element_version` exists in
/// the accumulator whose root hash is `expected_root_hash` using the provided proof.
pub fn verify_tx_state(
    tx_info: &TransactionInfoWithProof,
    expected_root_hash: HashValue,
    element_index: u64,
) -> Result<(), Error> {
    let element_hash = hash_tx_info(&tx_info.transaction_info);
    let proof = &tx_info.ledger_info_to_transaction_info_proof;
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

pub fn verify_existence_proof(
    proof: &[u8],
    expected_root_hash: HashValue,
    element_key: HashValue,
    element_hash: HashValue,
) -> Result<(), Error> {
    let proof = SparseMerkleProof::decode_as::<Proto>(proof).unwrap();

    if proof.siblings.len() > HashValue::LENGTH_IN_BITS {
        // "Sparse Merkle Tree proof has more than {} ({} + {}) siblings.",
        return Err(StorageVerificationError::MaxSiblingsExceeded(
            HashValue::LENGTH_IN_BITS,
            proof.siblings.len(),
        )
        .into());
    }

    let Some(leaf) = proof.leaf else {
        return Err(StorageVerificationError::ExpectedMembershipVerification.into());
    };

    // This is an inclusion proof, so the key and value hash provided in the proof
    // should match element_key and element_value_hash. `siblings` should prove the
    // route from the leaf node to the root.
    if element_key != leaf.key {
        return Err(StorageVerificationError::LeafKeyMismatch(element_key, leaf.key).into());
        //     Key in proof: {:x}. Expected key: {:x}. \
        // Element hash: {:x}. Value hash in proof {:x}",
    }
    if element_hash != leaf.value_hash {
        return Err(
            StorageVerificationError::LeafValueMismatch(element_hash, leaf.value_hash).into(),
        );
    }

    let current_hash = proof.leaf.map_or(SPARSE_MERKLE_PLACEHOLDER_HASH, |leaf| {
        hash_sparse_merkle_leaf_node(&leaf)
    });
    let actual_root_hash = proof
        .siblings
        .iter()
        .rev()
        .zip(
            element_key
                .iter_bits()
                .rev()
                .skip(HashValue::LENGTH_IN_BITS - proof.siblings.len()),
        )
        .fold(current_hash, |hash, (sibling_hash, bit)| {
            if bit {
                SparseMerkleInternalNode::new(*sibling_hash, hash).hash()
            } else {
                SparseMerkleInternalNode::new(hash, *sibling_hash).hash()
            }
        });

    if actual_root_hash != expected_root_hash {
        return Err(StorageVerificationError::RootHashMismatch(
            actual_root_hash,
            expected_root_hash,
        )
        .into());
    }

    Ok(())
}

fn hash_tx_info(tx_info: &TransactionInfo) -> HashValue {
    let mut state = Sha3_256::new();
    state.update(
        Sha3_256::new()
            .chain_update("APTOS::TransactionInfo")
            .finalize(),
    );
    bcs::serialize_into(&mut state, tx_info).expect("expected to be able to serialize");
    HashValue(state.finalize().into())
}

fn hash_sparse_merkle_leaf_node(leaf: &SparseMerkleLeafNode) -> HashValue {
    let mut state = Sha3_256::new();
    state.update(
        Sha3_256::new()
            .chain_update("APTOS::SparseMerkleLeafNode")
            .finalize(),
    );
    state.update(leaf.key.as_ref());
    state.update(leaf.value_hash.as_ref());
    HashValue(state.finalize().into())
}

fn hash_inner_node(left_child: HashValue, right_child: HashValue) -> HashValue {
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
