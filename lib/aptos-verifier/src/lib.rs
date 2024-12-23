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
    hash::{BytesBitIterator, LittleEndian, H256},
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
    println!("{element_key:?}");

    println!("{}", proof.siblings.len());
    let actual_root_hash = proof
        .siblings
        .iter()
        .rev()
        .zip(
            BytesBitIterator::<LittleEndian>::new(&element_key)
                .rev()
                .skip(256 - proof.siblings.len()),
        )
        .fold(current_hash, |hash, (sibling_hash, bit)| {
            println!("bit: {bit}");
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

#[cfg(test)]
pub mod tests {
    use hex_literal::hex;
    use unionlabs::{
        aptos::sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        hash::H256,
    };

    use crate::verify_existence_proof;

    #[test]
    fn verify_existence_passes() {
        verify_existence_proof(
            SparseMerkleProof {
                leaf: Some(SparseMerkleLeafNode {
                    key: H256::new(hex!(
                        "f2d067d8ef7e97deb231d46f40f9f30200e6f1dad495d33e2a7911825a97ad14"
                    )),
                    value_hash: H256::new(hex!(
                        "40414333f8109f8cb971c67c9eca3c0049e21e6c5e28551f1a4975c96ab15212"
                    )),
                }),
                siblings: vec![
                    H256::new(hex!(
                        "fafdceaec25fd64517ce3745992467dfac306a5ce59e63255da5b9f58d1417ea"
                    )),
                    H256::new(hex!(
                        "4480c449082954642653a4570c7cb2ea2114d79b61621b94f095f25d640b6e27"
                    )),
                    H256::new(hex!(
                        "0fc055434d70262945d428a5eda3d8396aa960c65ee8c4e79bd20638a95e7a31"
                    )),
                    H256::new(hex!(
                        "731e28eb6655e01b8714aa72f76a0f468c330b46eb9c21816a88e56840896f24"
                    )),
                    H256::new(hex!(
                        "b120265e60289e6e44216efd4f3fba86a8de645d3eb7912ff09812024c639b2f"
                    )),
                    H256::new(hex!(
                        "d90e0a63c7c3cf7ed000841a85f981d8c6bec4c23353822204c7c9e9c5dee4db"
                    )),
                    H256::new(hex!(
                        "30b21a8a3bf202b5fe18e415c299fd3b9985462a6292fffdabd5b32fbc27ba30"
                    )),
                    H256::new(hex!(
                        "5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000"
                    )),
                    H256::new(hex!(
                        "5b9096922002407577b4e46e6466aadb15cbb9521fd0e9847d474398ea3736e2"
                    )),
                    H256::new(hex!(
                        "5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000"
                    )),
                    H256::new(hex!(
                        "884f8b72a832aa718c6590d0bfeb1ec85b546611b6c07f8df563827974ad8134"
                    )),
                ],
            },
            hex!("02388da3aee85236d64e272fec0b1a6fcd4962986327971faef9ee2951a4ad6a"),
            hex!("f2d067d8ef7e97deb231d46f40f9f30200e6f1dad495d33e2a7911825a97ad14"),
            hex!("40414333f8109f8cb971c67c9eca3c0049e21e6c5e28551f1a4975c96ab15212"),
        )
        .unwrap();
    }
}
