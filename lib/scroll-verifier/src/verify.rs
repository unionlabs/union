use core::fmt::Debug;

use ethereum_verifier::{verify_account_storage_root, verify_storage_proof};
use ethers_core::abi::{AbiDecode, AbiError};
use scroll_codec::CommitBatchError;
use sha3::Digest;
use unionlabs::{
    hash::{H160, H256},
    ibc::lightclients::scroll::{client_state::ClientState, header::Header},
    scroll::account::Account,
    uint::U256,
};
use zktrie::{decode_smt_proofs, Byte32, Database, Hash, MemDB, PoseidonHash, TrieData, ZkTrie};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error(transparent)]
    InvalidContractAddressProof(ethereum_verifier::Error),
    #[error("{0}")]
    InvalidRollupProof(ethereum_verifier::Error),
    #[error("invalid zktrie")]
    ZkTrie(zktrie::Error),
    #[error("node value mismatch")]
    ValueMismatch,
    #[error("unable to decode commit batch calldata")]
    CommitBatchCallDecode(#[from] AbiError),
    #[error("error while calculating batch hash")]
    CommitBatch(#[from] CommitBatchError),
}

pub fn verify_header(
    client_state: ClientState,
    header: Header,
    l1_state_root: H256,
) -> Result<(), Error> {
    // Verify that the rollup account root is part of the L1 root
    verify_account_storage_root(
        l1_state_root,
        &client_state.rollup_contract_address,
        &header.l1_account_proof.proof,
        &header.l1_account_proof.storage_root,
    )
    .map_err(Error::InvalidContractAddressProof)?;

    // Verify that the latest batch index is part of the rollup account root
    verify_storage_proof(
        header.l1_account_proof.storage_root,
        client_state.latest_batch_index_slot,
        &rlp::encode(&header.last_batch_index),
        &header.last_batch_index_proof.proofs[0].proof,
    )
    .map_err(Error::InvalidRollupProof)?;

    // Verify that the rollup finalized state root is part of the rollup account root
    verify_storage_proof(
        header.l1_account_proof.storage_root,
        batch_index_mapping_key(
            client_state.rollup_finalized_state_roots_slot,
            header.last_batch_index.into(),
        ),
        &rlp::encode(&U256::from_be_bytes(header.l2_state_root.into())),
        &header.l2_state_proof.proofs[0].proof,
    )
    .map_err(Error::InvalidRollupProof)?;

    let batch_hash = scroll_codec::commit_batch(
        <scroll_codec::CommitBatchCall as AbiDecode>::decode(header.commit_batch_calldata)?,
        header.blob_versioned_hash,
        header.l1_message_hashes,
    )?;

    // Verify that the batch hash is part of the rollup account root
    verify_storage_proof(
        header.l1_account_proof.storage_root,
        batch_index_mapping_key(
            client_state.rollup_committed_batches_slot,
            header.last_batch_index.into(),
        ),
        &rlp::encode(&U256::from_be_bytes(batch_hash.into())),
        &header.batch_hash_proof.proofs[0].proof,
    )
    .map_err(Error::InvalidRollupProof)?;

    // Verify that the ibc account root is part of the rollup root
    scroll_verify_zktrie_account_storage_root(
        header.l2_state_root,
        &client_state.ibc_contract_address,
        &header.l2_ibc_account_proof.proof,
        &header.l2_ibc_account_proof.storage_root,
    )?;
    Ok(())
}

/// Storage slot of a `mapping(uint256 => bytes32)` mapping, where the mapping is at slot `slot` and the `uint256` is the `batch_index`.
pub fn batch_index_mapping_key(slot: U256, batch_index: U256) -> U256 {
    U256::from_be_bytes(
        sha3::Keccak256::new()
            .chain_update(batch_index.to_be_bytes())
            .chain_update(slot.to_be_bytes())
            .finalize()
            .into(),
    )
}

pub fn get_zktrie_node(
    root: H256,
    key: impl AsRef<[u8]>,
    proof: &[impl AsRef<[u8]>],
) -> Result<TrieData<PoseidonHash>, Error> {
    let mut db = MemDB::<PoseidonHash>::default();
    for raw_proof in proof.iter() {
        if let Some(node) = decode_smt_proofs(raw_proof.as_ref()).map_err(Error::ZkTrie)? {
            db.update_node(node).map_err(Error::ZkTrie)?;
        }
    }
    ZkTrie::<PoseidonHash>::new(256, Hash::from(Byte32::from(root.0)))
        .get_data(&mut db, key.as_ref())
        .map_err(Error::ZkTrie)
}

pub fn verify_zktrie_storage_proof(
    root: H256,
    key: H256,
    expected_value: &[u8],
    proof: &[impl AsRef<[u8]>],
) -> Result<(), Error> {
    match get_zktrie_node(root, key.as_ref(), proof)? {
        TrieData::Node(node) if node.data() == expected_value => Ok(()),
        _ => Err(Error::ValueMismatch),
    }
}

pub fn verify_zktrie_storage_absence(
    root: H256,
    key: H256,
    proof: &[impl AsRef<[u8]>],
) -> Result<(), Error> {
    match get_zktrie_node(root, key.as_ref(), proof)? {
        TrieData::NotFound => Ok(()),
        _ => Err(Error::ValueMismatch),
    }
}

pub fn scroll_verify_zktrie_account_storage_root(
    root: H256,
    address: &H160,
    proof: &[impl AsRef<[u8]>],
    expected_storage_root: &H256,
) -> Result<(), Error> {
    match get_zktrie_node(root, address.as_ref(), proof)? {
        TrieData::NotFound => Err(Error::ValueMismatch),
        TrieData::Node(node) => {
            let account = Account::decode(node.data()).map_err(|_| Error::ValueMismatch)?;
            if &account.storage_root == expected_storage_root {
                Ok(())
            } else {
                Err(Error::ValueMismatch)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::{
        hash::{H160, H256},
        ibc::{
            core::client::height::Height,
            lightclients::{
                ethereum::proof::Proof,
                scroll::{client_state::ClientState, header::Header},
            },
        },
    };

    use crate::{verify_header, verify_zktrie_storage_absence, verify_zktrie_storage_proof};

    #[test]
    fn test_update_header() {
        let scroll_client_state = ClientState {
            l1_client_id: "cometbls-1".to_string(),
            chain_id: 534351.into(),
            latest_slot: 65327,
            latest_batch_index_slot: 156.into(),
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            rollup_contract_address: H160(hex!("2d567ece699eabe5afcd141edb7a4f2d0d6ce8a0")),
            rollup_finalized_state_roots_slot: 158.into(),
            rollup_committed_batches_slot: 157.into(),
            ibc_contract_address: H160(hex!("0000000000000000000000000000000000000000")),
            ibc_commitment_slot: 0.into(),
        };
        let scroll_header: Header =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_header.json").unwrap())
                .unwrap();

        let l1_state_root = H256(hex!(
            "4d47173201f8ded2c250d7f7f572a22d13061ed83009f451d271e0fabfa44425"
        ));

        assert!(matches!(
            verify_header(scroll_client_state, scroll_header, l1_state_root),
            Ok(())
        ));
    }

    #[test]
    fn test_l2_contract_slot_exist() {
        let proof: Proof =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_proof.json").unwrap())
                .unwrap();
        assert_eq!(
            verify_zktrie_storage_proof(
                H256(hex!(
                    "1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0"
                )),
                proof.key.to_be_bytes().into(),
                &proof.value.to_be_bytes(),
                &proof.proof
            ),
            Ok(())
        )
    }

    #[test]
    fn test_l2_contract_slot_absent() {
        let proof: Proof =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_absent.json").unwrap())
                .unwrap();
        assert_eq!(
            verify_zktrie_storage_absence(
                H256(hex!(
                    "1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0"
                )),
                proof.key.to_be_bytes().into(),
                &proof.proof
            ),
            Ok(())
        )
    }
}
