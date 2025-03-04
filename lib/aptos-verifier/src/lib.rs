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
        ledger_info::{LedgerInfo, LedgerInfoWithSignatures, LedgerInfoWithV0},
        sparse_merkle_proof::{SparseMerkleLeafNode, SparseMerkleProof},
        state_proof::StateProof,
        storage_proof::StateValue,
        transaction_info::TransactionInfo,
        transaction_proof::TransactionInfoWithProof,
        validator_verifier::ValidatorVerifier,
    },
    primitives::{encoding::HexPrefixed, H256, H384, H768},
    BytesBitIteratorBE,
};

pub(crate) const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
// "SPARSE_MERKLE_PLACEHOLDER_HASH"
pub(crate) const SPARSE_MERKLE_PLACEHOLDER_HASH: [u8; 32] =
    hex!("00005350415253455F4D45524B4C455F504C414345484F4C4445525F48415348");

pub trait BlsVerify {
    fn verify_signature<'pk>(
        &self,
        public_keys: impl IntoIterator<Item = &'pk H384>,
        msg: &[u8],
        signature: H768,
    ) -> Result<(), Error>;
}

/// Given a `trusted_state`, verify a state transition using `state_proof`
///
/// * `current_validator_verifier`: The validator verifier for the current(trusted) epoch.
/// * `trusted_state`: Currently trusted `LedgerInfo`. Note that if there's any epoch change, it **MUST** start
///    from the current epoch + 1.
/// * `state_proof`: Proof of state transition. Note that the function expects epoch changes to be in an ascending
///    order respective to the epoch number.
/// * `bls_verifier`: BLS verifier
pub fn verify_state_proof<V: BlsVerify>(
    current_validator_verifier: &ValidatorVerifier,
    trusted_state: &LedgerInfo,
    state_proof: &StateProof,
    bls_verifier: &V,
) -> Result<(), Error> {
    let mut current_epoch = trusted_state.commit_info.epoch;
    // TODO(aeryz): does this info exist on every block info?
    let mut current_next_verifier = trusted_state.commit_info.next_epoch_state.as_ref().unwrap();

    for li in &state_proof.epoch_changes.ledger_info_with_sigs {
        let LedgerInfoWithSignatures::V0(li) = li;
        if current_epoch + 1 != li.ledger_info.commit_info.epoch {
            println!("{} {}", current_epoch, li.ledger_info.commit_info.epoch);
            return Err(Error::InvalidEpochOrder);
        }

        verify_ledger_info(&current_next_verifier.verifier, li, bls_verifier)?;

        current_epoch += 1;
        current_next_verifier = li
            .ledger_info
            .commit_info
            .next_epoch_state
            .as_ref()
            .unwrap();
    }

    let LedgerInfoWithSignatures::V0(li) = &state_proof.latest_li_w_sigs;

    if li.ledger_info.commit_info.epoch == current_epoch {
        verify_ledger_info(current_validator_verifier, li, bls_verifier)
    } else {
        verify_ledger_info(&current_next_verifier.verifier, li, bls_verifier)
    }
}

pub fn verify_ledger_info<V: BlsVerify>(
    validator_verifier: &ValidatorVerifier,
    ledger_info: &LedgerInfoWithV0,
    bls_verifier: &V,
) -> Result<(), Error> {
    // Self::check_num_of_voters(self.len() as u16, multi_signature.get_signers_bitvec())?;
    let (pub_keys, _) = BytesBitIteratorBE::new(&ledger_info.signatures.validator_bitmask.inner)
        .enumerate()
        .filter(|(_, is_true)| *is_true)
        .map(|(i, _)| {
            let validator = validator_verifier.validator_infos.get(i).unwrap();
            (
                H384::<HexPrefixed>::new(
                    validator.public_key.pubkey.as_slice().try_into().unwrap(),
                ),
                validator.address,
            )
        })
        .collect::<(Vec<_>, Vec<_>)>();

    // self.check_voting_power(authors.iter(), true)?;
    bls_verifier.verify_signature(
        &pub_keys,
        &hash_ledger_info(&ledger_info.ledger_info),
        ledger_info.signatures.sig.unwrap(),
    )
}
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
            BytesBitIteratorBE::new(&element_key)
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

fn hash_ledger_info(ledger_info: &LedgerInfo) -> Vec<u8> {
    let mut buf = Sha3_256::new()
        .chain_update("APTOS::LedgerInfo")
        .finalize()
        .to_vec();
    bcs::serialize_into(&mut buf, ledger_info).expect("expected to be able to serialize");
    buf
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

    state.chain_update(buf).finalize().into()
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
mod tests {
    use cosmwasm_crypto::HashFunction;
    use unionlabs::{
        aptos::state_proof::StateProof,
        encoding::{DecodeAs, Json},
    };

    use super::*;

    struct BlsVerifier;

    pub const BLS12_381_G1_GENERATOR: [u8; 48] = [
        151, 241, 211, 167, 49, 151, 215, 148, 38, 149, 99, 140, 79, 169, 172, 15, 195, 104, 140,
        79, 151, 116, 185, 5, 161, 78, 58, 63, 23, 27, 172, 88, 108, 85, 232, 63, 249, 122, 26,
        239, 251, 58, 240, 10, 219, 34, 198, 187,
    ];

    impl BlsVerify for BlsVerifier {
        fn verify_signature<'pk>(
            &self,
            public_keys: impl IntoIterator<Item = &'pk H384>,
            msg: &[u8],
            signature: H768,
        ) -> Result<(), Error> {
            let pubkeys = public_keys
                .into_iter()
                .flat_map(|x| *x)
                .collect::<Vec<u8>>();

            let pubkey = cosmwasm_crypto::bls12_381_aggregate_g1(&pubkeys).unwrap();

            pub const DST_POP_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
            let hashed_msg =
                cosmwasm_crypto::bls12_381_hash_to_g2(HashFunction::Sha256, msg, DST_POP_G2);

            let valid = cosmwasm_crypto::bls12_381_pairing_equality(
                &BLS12_381_G1_GENERATOR,
                signature.as_ref(),
                &pubkey,
                &hashed_msg,
            )
            .unwrap();

            if valid {
                Ok(())
            } else {
                panic!("invalid signature");
            }
        }
    }

    #[derive(serde::Deserialize)]
    struct StateProofResponse {
        state_proof: StateProof,
    }

    #[test]
    fn test_aptos_full_verification() {
        let state_proof_response = r#"{"state_proof":{"latest_li_w_sigs":{"V0":{"ledger_info":{"commit_info":{"epoch":119,"round":16,"id":"922f07af4849dec266bda670b62925b11b5565f9bfc7179efbe9da7a53058724","executed_state_id":"b6b1c56ae0c373ccd07329e5e75ac39b81253407f37bf8a7914c1bac5be08d87","version":13266,"timestamp_usecs":1740872921079217,"next_epoch_state":null},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[128]},"sig":"0xaff12e7c24499a4b8122451f1eff007fd99e1412aa3717c4e9e7fb3bebc6e5191e961c69840851f51cc3707499ed1ebf049ee47d3cbc5fe1ac9f58ee124d26b3039c1bf3fa360c9cccd590f20afbb5f058c23631cd39689543f71d3998344681"}}},"epoch_changes":{"ledger_info_with_sigs":[{"V0":{"ledger_info":{"commit_info":{"epoch":116,"round":58,"id":"312707bf0195fcc81771e572e275def4ce8fa37c234d2c4a378a260606c7afe2","executed_state_id":"0741c0bafc105474169e9bf4e0be1f23a4f846cb03c20984ca843d2720b3f7bf","version":13007,"timestamp_usecs":1740871950517277,"next_epoch_state":{"epoch":117,"verifier":{"validator_infos":[{"address":"d2982978cec002dfb4b3b7921de20b828e82e20ad5600d8989cd58150bb2a15c","public_key":"0x8f31062af1a3eddcaebd429a413c89b4dd7059c00a5355311a88ac9a0691868a40c236397456433e7a98a5f32ef3c43b","voting_power":10}]}}},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[128]},"sig":"0xb90b90e62ef1d13e2315346a11914485cb9ef57607541c2cb51ca96b5700f73764752c36b40fa138cde1063c2e7580ff17dd5dee04939c5f9cbac82b57bb23a97ca69bbc222b1a51bd24fb05318724d8f73400092381a656cbd6c2ac3853f87b"}}},{"V0":{"ledger_info":{"commit_info":{"epoch":117,"round":59,"id":"edb6d1cbb9e05e64a5e61453f941a8746ac51ea0b37277c84da791eb5772903c","executed_state_id":"6e22c941807bd825e192f7d32845b2e885ca446bfddecbec9d760fb5668c830b","version":13125,"timestamp_usecs":1740872013560533,"next_epoch_state":{"epoch":118,"verifier":{"validator_infos":[{"address":"d2982978cec002dfb4b3b7921de20b828e82e20ad5600d8989cd58150bb2a15c","public_key":"0x8f31062af1a3eddcaebd429a413c89b4dd7059c00a5355311a88ac9a0691868a40c236397456433e7a98a5f32ef3c43b","voting_power":10}]}}},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[128]},"sig":"0xae4180f98d41ba4adccc25f1f28d598e9909199fdc61a95f85796f175612a1051138ee3005b378982018582a8b799dc003021c03a72eb3f237d88de1cdf72e2afe95aa3b36cdeb48b197c8a310bf2fb2c60e381fce4a98cbd32571dc27bc9622"}}},{"V0":{"ledger_info":{"commit_info":{"epoch":118,"round":53,"id":"028253e6c5d782817d86019ef756965a257df7e5d1810ca975c5228ba228f0b3","executed_state_id":"444b8fc3c16d5ade2ee42be83d89c075716a2d34c0065432d45b4db3db6aa1d3","version":13233,"timestamp_usecs":1740872906587940,"next_epoch_state":{"epoch":119,"verifier":{"validator_infos":[{"address":"d2982978cec002dfb4b3b7921de20b828e82e20ad5600d8989cd58150bb2a15c","public_key":"0x8f31062af1a3eddcaebd429a413c89b4dd7059c00a5355311a88ac9a0691868a40c236397456433e7a98a5f32ef3c43b","voting_power":10}]}}},"consensus_data_hash":"0000000000000000000000000000000000000000000000000000000000000000"},"signatures":{"validator_bitmask":{"inner":[128]},"sig":"0xa7c4d1ee4a05f51848c314cd6a13686cea1e3affb0bfeb7498a1d57ec671228f1bf6807efd52d42a74af6133978c562013354227ea3db6bbbb20f79e940307b144d2cc08b7281685a9ac2e890aa8f497cf514ef13ebc9e3998eac3e87ed7e8d5"}}}],"more":false}},"tx_proof":{"ledger_info_to_transaction_info_proof":{"siblings":["a4140f7e236a14395e65d95295b9535d407aff135bf963ef0ed3ed7337aa72af","47b822eb4b68fef10ce704b310935444dcc89864aeecbb7ef8071f0b0efe9f8d","3a96143203411f9d28c8307fc5cab7356d6ba46334fdc11f4808b12941966bf7","5556b583f5ab11a8d436d50319e44e3d9f625bbb22381a4936503cfd2720d552","cce2679849ffe8474343d45f41f4528d19f941a0fa136175a1e0b2d1a8aa3a5e","a78d9e3fd2325fe5a7f6d4f2635ec5627b8b301539b2942eeb81073a64375e95","8c5e8355ca3f4a9024f0a780bcfa2c1c21099ebf87a7ab6d4290ab4ea8c0cdeb","496ac967737b55026d557a6ccd4a124719285a6c6f6fd0cd24b7ff07e90090a1","8d5c5c1b03cdd95d8cb3f19e329030ba3d8f1f25fb4937c914016074383ed6f5","87797c1f2cf51bac9b422c01e7a08e39e303f734004f618ce9c7433f31679556","a6c04e9e97a5a74e2dad96d2aa3a253f32e91d51860da9b879a6471b5f3e9409","d80171fe08006e630363ac73b42961cfa95d89e4f0d0e39f1a941f5b8595a87b","d182842f579a375d0dce4859fb85f60480fd8dfbc495305caf6d0c66d3e4ab72","9df97297843ca0b9ffccf3ff3a09d08aea2fd18f21a5cc26ad86a0b4ca4493ba"]},"transaction_info":{"V0":{"gas_used":0,"status":"Success","transaction_hash":"bd6bf5bbfea15b145526fc3f050d1723533da31a6163cc9f5da410a2eb0e58c0","event_root_hash":"414343554d554c41544f525f504c414345484f4c4445525f4841534800000000","state_change_hash":"afb6e14fe47d850fd0a7395bcfb997ffacf4715e0f895cc162c218e4a7564bc6","state_checkpoint_hash":"5985147d09cefc857cbf837010c4465c4f0940440270a1ef36c3abda36bf23af","state_cemetery_hash":null}}},"tx_index":7117}"#;
        let mut state_proof_response =
            StateProofResponse::decode_as::<Json>(state_proof_response.as_bytes()).unwrap();

        let initial_state = state_proof_response
            .state_proof
            .epoch_changes
            .ledger_info_with_sigs
            .iter()
            .skip(1)
            .next()
            .cloned()
            .unwrap();

        let current_validator_verifier = state_proof_response
            .state_proof
            .epoch_changes
            .ledger_info_with_sigs
            .first()
            .unwrap()
            .ledger_info()
            .commit_info
            .next_epoch_state
            .as_ref()
            .unwrap()
            .clone();

        state_proof_response
            .state_proof
            .epoch_changes
            .ledger_info_with_sigs = state_proof_response
            .state_proof
            .epoch_changes
            .ledger_info_with_sigs[2..]
            .to_vec();

        verify_state_proof(
            &current_validator_verifier.verifier,
            initial_state.ledger_info(),
            &state_proof_response.state_proof,
            &BlsVerifier,
        )
        .unwrap();
    }
}
