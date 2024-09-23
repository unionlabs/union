// TODO: hasher.chain_update() can be used throughout this file

pub mod error;

use std::io::Write as _;

pub use error::Error;
use error::StorageVerificationError;
use hex_literal::hex;
use sha3::{Digest, Sha3_256};
use unionlabs::{
    aptos::{
        account::AccountAddress,
        sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        storage_proof::StateValue,
        transaction_info::TransactionInfo,
        transaction_proof::TransactionInfoWithProof,
    },
    hash::{BytesBitIterator, H256},
};

pub(crate) const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
// "SPARSE_MERKLE_PLACEHOLDER_HASH"
pub(crate) const SPARSE_MERKLE_PLACEHOLDER_HASH: [u8; 32] =
    hex!("00005350415253455F4D45524B4C455F504C414345484F4C4445525F48415348");

/// Verifies an element whose hash is `element_hash` and version is `element_version` exists in
/// the accumulator whose root hash is `expected_root_hash` using the provided proof.
pub fn verify_tx_state(
    tx_info: &TransactionInfoWithProof,
    expected_root_hash: [u8; 32],
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
                        hash_inner_node(hash, *sibling_hash.get())
                    } else {
                        // the current node is a right child.
                        hash_inner_node(*sibling_hash.get(), hash)
                    },
                    // The index of the parent at its level.
                    index / 2,
                )
            },
        )
        .0;

    if actual_root_hash != expected_root_hash {
        return Err(Error::RootHashMismatch {
            expected: H256::new(expected_root_hash),
            given: H256::new(actual_root_hash),
        });
    }

    Ok(())
}

pub fn verify_membership(
    proof: SparseMerkleProof,
    expected_root_hash: [u8; 32],
) -> Result<(), Error> {
    let Some(proof_leaf) = proof.leaf else {
        return Err(StorageVerificationError::ExpectedMembershipVerification.into());
    };

    verify_existence_proof(
        proof,
        expected_root_hash,
        proof_leaf.key.into(),
        proof_leaf.value_hash.into(),
    )
}

pub fn verify_existence_proof(
    proof: SparseMerkleProof,
    expected_root_hash: [u8; 32],
    element_key: [u8; 32],
    element_hash: [u8; 32],
) -> Result<(), Error> {
    if proof.siblings.len() > 256 {
        // "Sparse Merkle Tree proof has more than {} ({} + {}) siblings.",
        return Err(
            StorageVerificationError::MaxSiblingsExceeded(256, proof.siblings.len()).into(),
        );
    }

    let Some(leaf) = proof.leaf else {
        return Err(StorageVerificationError::ExpectedMembershipVerification.into());
    };

    // This is an inclusion proof, so the key and value hash provided in the proof
    // should match element_key and element_value_hash. `siblings` should prove the
    // route from the leaf node to the root.
    if &element_key != leaf.key.get() {
        return Err(StorageVerificationError::LeafKeyMismatch(
            H256::new(element_key),
            H256::new(*leaf.key.get()),
        )
        .into());
    }

    if &element_hash != leaf.value_hash.get() {
        return Err(StorageVerificationError::LeafValueMismatch(
            H256::new(element_hash),
            H256::new(*leaf.value_hash.get()),
        )
        .into());
    }

    let current_hash = proof.leaf.map_or(SPARSE_MERKLE_PLACEHOLDER_HASH, |leaf| {
        hash_sparse_merkle_leaf_node(&leaf)
    });
    let actual_root_hash = proof
        .siblings
        .iter()
        .rev()
        .zip(
            BytesBitIterator::new(&element_key)
                .rev()
                .skip(256 - proof.siblings.len()),
        )
        .fold(current_hash, |hash, (sibling_hash, bit)| {
            if bit {
                SparseMerkleInternalNode::new(*sibling_hash.get(), hash).hash()
            } else {
                SparseMerkleInternalNode::new(hash, *sibling_hash.get()).hash()
            }
        });

    if actual_root_hash != expected_root_hash {
        return Err(StorageVerificationError::RootHashMismatch(
            H256::new(actual_root_hash),
            H256::new(expected_root_hash),
        )
        .into());
    }

    Ok(())
}

pub fn hash_state_value(value: &StateValue) -> [u8; 32] {
    Sha3_256::new()
        .chain_update(Sha3_256::new().chain_update("APTOS::StateValue").finalize())
        .chain_update(bcs::to_bytes(value).expect("cannot fail"))
        .finalize()
        .into()
}

pub fn hash_table_key(key: &[u8], table_handle: &AccountAddress) -> [u8; 32] {
    // TODO(aeryz): make this a const
    let mut buf = vec![1];
    bcs::serialize_into(&mut buf, &table_handle).unwrap();
    buf.write_all(key).unwrap();

    Sha3_256::new()
        .chain_update(Sha3_256::new().chain_update("APTOS::StateKey").finalize())
        .chain_update(&buf)
        .finalize()
        .into()
}

fn hash_tx_info(tx_info: &TransactionInfo) -> [u8; 32] {
    let mut state = Sha3_256::new();
    state.update(
        Sha3_256::new()
            .chain_update("APTOS::TransactionInfo")
            .finalize(),
    );
    bcs::serialize_into(&mut state, tx_info).expect("expected to be able to serialize");

    state.finalize().into()
}

fn hash_sparse_merkle_leaf_node(leaf: &SparseMerkleLeafNode) -> [u8; 32] {
    let mut state = Sha3_256::new();
    state.update(
        Sha3_256::new()
            .chain_update("APTOS::SparseMerkleLeafNode")
            .finalize(),
    );
    state.update(leaf.key.as_ref());
    state.update(leaf.value_hash.as_ref());
    state.finalize().into()
}

fn hash_inner_node(left_child: [u8; 32], right_child: [u8; 32]) -> [u8; 32] {
    let mut state = Sha3_256::new();
    state.update(
        Sha3_256::new()
            .chain_update("APTOS::TransactionAccumulator")
            .finalize(),
    );
    state.update(left_child.as_ref());
    state.update(right_child.as_ref());
    state.finalize().into()
}

pub struct SparseMerkleInternalNode {
    left_child: [u8; 32],
    right_child: [u8; 32],
}

impl SparseMerkleInternalNode {
    pub fn new(left_child: [u8; 32], right_child: [u8; 32]) -> Self {
        Self {
            left_child,
            right_child,
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut state = Sha3_256::new();
        state.update(
            Sha3_256::new()
                .chain_update("APTOS::SparseMerkleInternal")
                .finalize(),
        );
        state.update(self.left_child.as_ref());
        state.update(self.right_child.as_ref());
        state.finalize().into()
    }
}
