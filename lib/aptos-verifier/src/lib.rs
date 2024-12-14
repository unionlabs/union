pub mod error;

use std::io::Write;

pub use error::Error;
use error::StorageVerificationError;
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

const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
const SPARSE_MERKLE_PLACEHOLDER_HASH: [u8; 32] = hex_literal::hex!(
    "00005350415253455F4D45524B4C455F504C414345484F4C4445525F48415348"
);

/// Verifies an element's existence in the accumulator with the given proof.
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

    let actual_root_hash = proof.siblings.iter().fold(
        (element_hash, element_index),
        |(hash, index), sibling_hash| {
            let parent_hash = if index % 2 == 0 {
                hash_inner_node(hash, *sibling_hash.get())
            } else {
                hash_inner_node(*sibling_hash.get(), hash)
            };
            (parent_hash, index / 2)
        },
    ).0;

    if actual_root_hash != expected_root_hash {
        return Err(Error::RootHashMismatch {
            expected: H256::new(expected_root_hash),
            given: H256::new(actual_root_hash),
        });
    }

    Ok(())
}

/// Verifies membership proof in a Sparse Merkle Tree.
pub fn verify_membership(
    proof: SparseMerkleProof,
    expected_root_hash: [u8; 32],
) -> Result<(), Error> {
    let proof_leaf = proof.leaf.ok_or_else(|| {
        StorageVerificationError::ExpectedMembershipVerification.into()
    })?;

    let element_key = hash_table_key(key, &table_handle);
    let element_value_hash = hash_state_value(&state_value);

    verify_existence_proof(proof, expected_root_hash, element_key, element_value_hash)
}

/// Verifies the existence of an element in a Sparse Merkle Tree.
pub fn verify_existence_proof(
    proof: SparseMerkleProof,
    expected_root_hash: [u8; 32],
    element_key: [u8; 32],
    element_hash: [u8; 32],
) -> Result<(), Error> {
    if proof.siblings.len() > 256 {
        return Err(StorageVerificationError::MaxSiblingsExceeded(256, proof.siblings.len()).into());
    }

    let leaf = proof.leaf.ok_or_else(|| {
        StorageVerificationError::ExpectedMembershipVerification.into()
    })?;

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

    let current_hash = proof
        .leaf
        .map_or(SPARSE_MERKLE_PLACEHOLDER_HASH, hash_sparse_merkle_leaf_node);

    let actual_root_hash = proof.siblings.iter().rev().zip(
        BytesBitIterator::new(&element_key)
            .rev()
            .skip(256 - proof.siblings.len()),
    ).fold(current_hash, |hash, (sibling_hash, bit)| {
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

/// Computes the hash of a transaction's state value.
pub fn hash_state_value(value: &StateValue) -> [u8; 32] {
    hash_with_prefix("APTOS::StateValue", &bcs::to_bytes(value).unwrap())
}

/// Computes the hash of a table key.
pub fn hash_table_key(key: &[u8], table_handle: &AccountAddress) -> [u8; 32] {
    let mut buf = vec![1];
    bcs::serialize_into(&mut buf, table_handle).unwrap();
    buf.extend_from_slice(key);
    hash_with_prefix("APTOS::StateKey", &buf)
}

/// Helper to hash data with a specific prefix.
fn hash_with_prefix(prefix: &str, data: &[u8]) -> [u8; 32] {
    Sha3_256::new()
        .chain_update(Sha3_256::new().chain_update(prefix).finalize())
        .chain_update(data)
        .finalize()
        .into()
}

fn hash_tx_info(tx_info: &TransactionInfo) -> [u8; 32] {
    hash_with_prefix("APTOS::TransactionInfo", &bcs::to_bytes(tx_info).unwrap())
}

fn hash_sparse_merkle_leaf_node(leaf: &SparseMerkleLeafNode) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(
        Sha3_256::new()
            .chain_update("APTOS::SparseMerkleLeafNode")
            .finalize(),
    );
    hasher.update(leaf.key.as_ref());
    hasher.update(leaf.value_hash.as_ref());
    hasher.finalize().into()
}

fn hash_inner_node(left_child: [u8; 32], right_child: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(
        Sha3_256::new()
            .chain_update("APTOS::TransactionAccumulator")
            .finalize(),
    );
    hasher.update(left_child.as_ref());
    hasher.update(right_child.as_ref());
    hasher.finalize().into()
}

/// Represents an internal node in a Sparse Merkle Tree.
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
        hash_inner_node(self.left_child, self.right_child)
    }
}
