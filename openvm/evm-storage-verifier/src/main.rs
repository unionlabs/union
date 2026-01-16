#![cfg_attr(not(test), no_std, no_main)]

use alloc::vec::Vec;

use openvm::io::read;
use serde::{Deserialize, Serialize};
use unionlabs_primitives::{Bytes, H160, H256, U256};

extern crate alloc;

openvm::entry!(main);

#[derive(Debug, Serialize, Deserialize)]
pub struct Proof {
    pub state_root: H256,
    pub address: H160,
    pub storage_hash: H256,
    pub account_proof: Vec<Vec<u8>>,
    pub storage_proof: Vec<Vec<u8>>,
    pub key: U256,
    pub value: U256,
}

fn main() {
    let proof = read::<Proof>();

    evm_storage_verifier::verify_account_storage_root(
        proof.state_root,
        &proof.address,
        &proof.account_proof,
        &proof.storage_hash,
    )
    .unwrap();

    evm_storage_verifier::verify_storage_proof(
        proof.storage_hash,
        proof.key,
        proof.value,
        &proof.storage_proof,
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use alloy::rpc::types::EIP1186AccountProofResponse;
    use unionlabs_primitives::{FixedBytes, encoding::HexUnprefixed};

    use super::*;

    #[test]
    fn proof_to_word_serialized() {
        let proof = serde_json::from_slice::<EIP1186AccountProofResponse>(
            &std::fs::read(
                "../../lib/evm-storage-verifier/src/test/valid_storage_proof_sepolia.json",
            )
            .unwrap(),
        )
        .unwrap();

        let bz = openvm::serde::to_vec(&Proof {
            state_root: alloy::hex!(
                "545e7cf676baca0fad067f9884fbb2a42090c0fa63a00c217c60688917deee6e"
            )
            .into(),
            address: proof.address.0.into(),
            storage_hash: proof.storage_hash.into(),
            account_proof: proof.account_proof.into_iter().map(Into::into).collect(),
            key: U256::from_be_bytes(proof.storage_proof[0].key.as_b256().0),
            value: proof.storage_proof[0].value.into(),
            storage_proof: proof.storage_proof[0]
                .clone()
                .proof
                .into_iter()
                .map(Into::into)
                .collect(),
        })
        .unwrap();

        let mut out = "0x01".to_owned();

        for word in bz {
            write!(
                &mut out,
                "{}",
                FixedBytes::<4, HexUnprefixed>::new(word.to_le_bytes())
            )
            .unwrap();
        }

        std::fs::write("../../input.hex", out).unwrap();
    }
}
