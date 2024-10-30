use core::fmt::Debug;

use ethereum_verifier::{verify_account_storage_root, verify_storage_proof};
use scroll_codec::{hash_batch, HashBatchError};
use scroll_light_client_types::{ClientState, Header};
use unionlabs::{
    ethereum::slot::{MappingKey, Slot},
    hash::{H160, H256},
    scroll::account::Account,
    uint::U256,
};
use zktrie::{decode_smt_proofs, Byte32, Database, Hash, MemDB, PoseidonHash, TrieData, ZkTrie};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    InvalidL1AccountProof(ethereum_verifier::error::Error),
    #[error(transparent)]
    InvalidLastBatchIndexProof(ethereum_verifier::error::Error),
    #[error(transparent)]
    InvalidL2FinalizedStateRootProof(ethereum_verifier::error::Error),
    #[error(transparent)]
    InvalidBatchHashProof(ethereum_verifier::error::Error),
    #[error(transparent)]
    ZkTrie(#[from] zktrie::Error),
    #[error("node value mismatch")]
    ValueMismatch,
    #[error(transparent)]
    HashBatch(#[from] HashBatchError),
}

// 1. rollupContractOnL1 ∈ L1Stateroot
// 2. lastBatchIndex ≡ rollupContractOnL1.lastBatchIndex
// 3. L2stateRoot ≡ rollupContractOnL1.finalized[lastBatchIndex]
// 4. batchHash ≡ rollupContractOnL1.batchHashes[lastBatchIndex]
// 5. ibcContractOnL2 ∈ L2StateRoot
pub fn verify_header(
    client_state: ClientState,
    header: Header,
    l1_state_root: H256,
) -> Result<(), Error> {
    // Verify that the rollup account root is part of the L1 root
    verify_account_storage_root(
        l1_state_root,
        &client_state.l2_contract_address,
        &header.l1_account_proof.proof,
        &header.l1_account_proof.storage_root,
    )
    .map_err(Error::InvalidL1AccountProof)?;

    // Verify that the latest batch index is part of the rollup account root
    verify_storage_proof(
        header.l1_account_proof.storage_root,
        client_state.latest_batch_index_slot,
        &rlp::encode(&header.last_batch_index_proof.value),
        &header.last_batch_index_proof.proof,
    )
    .map_err(Error::InvalidLastBatchIndexProof)?;

    // Verify that the rollup finalized state root is part of the rollup account root
    verify_storage_proof(
        header.l1_account_proof.storage_root,
        mapping_index_to_slot_key(
            client_state.l2_finalized_state_roots_slot,
            header.last_batch_index_proof.value,
        ),
        &rlp::encode(&header.l2_state_root_proof.value),
        &header.l2_state_root_proof.proof,
    )
    .map_err(Error::InvalidL2FinalizedStateRootProof)?;

    let batch_hash = hash_batch(header.batch_header)?;

    // Verify that the batch hash is part of the rollup account root
    verify_storage_proof(
        header.l1_account_proof.storage_root,
        mapping_index_to_slot_key(
            client_state.l2_committed_batches_slot,
            header.last_batch_index_proof.value,
        ),
        &rlp::encode(&U256::from_be_bytes(batch_hash.into())),
        &header.batch_hash_proof.proof,
    )
    .map_err(Error::InvalidBatchHashProof)?;

    // Verify that the ibc account root is part of the rollup root
    scroll_verify_zktrie_account_storage_root(
        header.l2_state_root_proof.value.to_be_bytes().into(),
        &client_state.ibc_contract_address,
        &header.l2_ibc_account_proof.proof,
        &header.l2_ibc_account_proof.storage_root,
    )?;

    Ok(())
}

/// Storage slot of a `mapping(uint256 => bytes32)` mapping, where the mapping is at slot `slot` and the `uint256` is the `batch_index`.
pub fn mapping_index_to_slot_key(slot: U256, batch_index: U256) -> U256 {
    Slot::Mapping(&Slot::Offset(slot), MappingKey::Uint256(batch_index)).slot()
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
    ZkTrie::<PoseidonHash>::new(256, Hash::from(Byte32::from(*root.get())))
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

// #[cfg(test)]
// mod tests {
//     use hex_literal::hex;
//     use scroll_light_client_types::{ClientState, Header};
//     use unionlabs::{
//         hash::{H160, H256},
//         ibc::{core::client::height::Height, lightclients::ethereum::storage_proof::StorageProof},
//     };

//     use crate::{verify_header, verify_zktrie_storage_absence, verify_zktrie_storage_proof};

//     #[test]
//     fn test_update_header() {
//         let scroll_client_state = ClientState {
//             l1_client_id: "cometbls-1".to_string(),
//             chain_id: 534351.into(),
//             latest_slot: 65327,
//             latest_batch_index_slot: 156.into(),
//             frozen_height: Height {
//                 revision_number: 0,
//                 revision_height: 0,
//             },
//             l2_contract_address: H160::new(hex!("2d567ece699eabe5afcd141edb7a4f2d0d6ce8a0")),
//             l2_finalized_state_roots_slot: 158.into(),
//             l2_committed_batches_slot: 157.into(),
//             // Dummy contract address for the sake of testing
//             ibc_contract_address: H160::new(hex!("0000000000000000000000000000000000000000")),
//             ibc_commitment_slot: 0.into(),
//         };
//         let scroll_header: Header =
//             serde_json::from_str(&std::fs::read_to_string("tests/scroll_header.json").unwrap())
//                 .unwrap();
//         let l1_state_root = H256::new(hex!(
//             "40ab3b90af84c30c31eb0fe9fc8cc5260b59f619d770706750ea3e474ca47c59"
//         ));
//         assert_eq!(
//             verify_header(scroll_client_state, scroll_header, l1_state_root),
//             Ok(())
//         );
//     }

//     #[test]
//     fn test_l2_contract_slot_exist() {
//         let proof: StorageProof =
//             serde_json::from_str(&std::fs::read_to_string("tests/scroll_proof.json").unwrap())
//                 .unwrap();
//         assert_eq!(
//             verify_zktrie_storage_proof(
//                 H256::new(hex!(
//                     "1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0"
//                 )),
//                 proof.key.to_be_bytes().into(),
//                 &proof.value.to_be_bytes(),
//                 &proof.proof
//             ),
//             Ok(())
//         )
//     }

//     #[test]
//     fn test_l2_contract_slot_absent() {
//         let proof: StorageProof =
//             serde_json::from_str(&std::fs::read_to_string("tests/scroll_absent.json").unwrap())
//                 .unwrap();
//         assert_eq!(
//             verify_zktrie_storage_absence(
//                 H256::new(hex!(
//                     "1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0"
//                 )),
//                 proof.key.to_be_bytes().into(),
//                 &proof.proof
//             ),
//             Ok(())
//         )
//     }
// }
