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
    primitives::H256,
    BytesBitIterator,
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
    let mut buf = vec![];
    bcs::serialize_into(&mut buf, tx_info).expect("expected to be able to serialize");
    state.update(buf);

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

#[test]
fn test_membership_verify_success() {
    let proof: SparseMerkleProof = serde_json::from_str(r#"{"leaf":{"key":"6e4b28d40f98a106a65163530924c0dcb40c1349d3aa915d108b4d6cfc1ddb19","value_hash":"58e7bf0a6dd26e946946402cf50941bccba0a1724a2977bc2ec0986195e816a0"},"siblings":["1948fd458b370b4af82d68717cc05a54b4def266f6abae1e145529b23c842605","3cb1f03efda960b4acb5a119a933d6fd1a564f6c79a2e51efce337d3b90ee9fc","90082b6b09c6b2c74e4516fc7d16021986a02be5a5d9aa4636e7e85a55efea81","610db0a0b5237112b545bd4104208f58bd6b3344af7cb61e93f0be0483fbae44","93bd6d3814c8fa0c7e48df0be69010b00a5b22bce90f57ff77aa55b02a69d077","cf363af71ff591de4a6d67208f90bf2181699c6b2fc29e70169c5e5a4cb0fe16","20a067a6b1d387dd485781167453b44d62a82a45455252e942e7843a260ba751","bc8796ddc7b4dc70c84d94a7b4f7dc2ddcabfef7329ae7613ce06777b3652a14","9330e423338bf02370352eec1a859aaab9d547051441d6ce8d323458eb9d7e9d","1395f60e26649eae6b28c0598de6d822cfd41eef25e90b71949d2238aeebcf2a","f36442530dba881665d55e6990b49168df3b797d636df4cb664aed6f40f8458d","24201164119e44b45a577c817cc7c2ba99804978aae5f4e6918caa71465fc7fe","78add101849c73e0e48378821b1b477bfa7c2e61ef143369911cad820a4ab415","2616b21adb5de351fa04288b5c2170e71397904dd214e4cb8c60dc5741703841","1bbc1d74fdd5224e55615f87a3b36d2d39219aee13a07d3c0bd10bf29167370e","4044e478843cdf4a38b253218e7f538528b4a39c8655eee4ba6e55599203b4c0","235ec5b74305e0ade563df16eda641913d9ee5bb2026d75fdaf5a723f476d904","40a939dcef4e0ffd81ffc922cef404b475d5466c7bb0524cf55468d6f404a087","dd7a69bab2eccfd5707afa03375fe848fe26d2167bacd12bf81992ff42177478","883ff5974348b49852978d1482bccee6dec20ea2c4e391528a1580ca0c9a135e","20db621a20d61d5d8d8e4d36491d456a4bc99ebf2645af2f31eb843386b25bc9","321e76b23dd4ad9514bcbdeac367b9a3e1306c4d6f6f151ae8fbdbebf4c19568","334b56ad8afe3475932da9e8b7f9fae0ce7adc971b05d56dce1e37c54daada61","881bd464a4e77cbe3b32076451ee6942a783903ce256f214e565ff08f0cbb749","0b9ff7a37b22ce2a49dc2dd38a119d4a3e2b768c2e0777cb069b4c7b9c592631"]}"#).unwrap();
    let expected_root_hash =
        hex_literal::hex!("4dd91f1331754b8a01bd1f471170311ff50b7cfc9c83a471d47d4060f2ed01b2");
    assert!(verify_existence_proof(
        proof.clone(),
        expected_root_hash,
        *proof.leaf.as_ref().unwrap().key.clone().get(),
        *proof.leaf.as_ref().unwrap().value_hash.clone().get(),
    )
    .is_ok());
}
