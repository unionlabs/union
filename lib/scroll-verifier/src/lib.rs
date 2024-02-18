use core::fmt::Debug;

use ethereum_verifier::{
    verify_account_storage_root, verify_storage_proof, VerifyAccountStorageRootError,
    VerifyStorageProofError,
};
use sha3::Digest;
use unionlabs::{
    hash::{H160, H256},
    ibc::lightclients::scroll::{client_state::ClientState, header::Header},
    uint::U256,
};
use zktrie::{decode_smt_proofs, Byte32, Database, Hash, MemDB, PoseidonHash, TrieData, ZkTrie};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("invalid rollup or ibc contract address")]
    InvalidContractAddress,
    #[error("{0}")]
    InvalidContractAddressProof(#[from] VerifyAccountStorageRootError),
    #[error("{0}")]
    InvalidRollupProof(#[from] VerifyStorageProofError),
    #[error("invalid zktrie")]
    ZkTrie(zktrie::Error),
    #[error("node value mismatch")]
    ValueMismatch,
}

pub fn scroll_verify_header(
    scroll_client_state: ClientState,
    scroll_header: Header,
    l1_state_root: H256,
) -> Result<(), Error> {
    // TODO: account_proof.contract_address should be removed entirely from ethereum LC
    if scroll_client_state.rollup_contract_address
        != scroll_header.l1_account_proof.contract_address
        || scroll_client_state.ibc_contract_address
            != scroll_header.ibc_account_proof.contract_address
    {
        Err(Error::InvalidContractAddress)
    } else {
        // Verify that the rollup account root is part of the L1 root
        verify_account_storage_root(
            l1_state_root,
            &scroll_client_state.rollup_contract_address,
            &scroll_header.l1_account_proof.proof,
            &scroll_header.l1_account_proof.storage_root,
        )?;
        // Verify that the rollup finalized state root is part of the rollup account root
        verify_storage_proof(
            scroll_header.l1_account_proof.storage_root,
            finalized_state_root_key(
                scroll_client_state.rollup_finalized_state_roots_slot,
                scroll_header.finalized_proof.batch_index.into(),
            ),
            &rlp::encode(&scroll_header.finalized_proof.finalized_state_root),
            &scroll_header.finalized_proof.proof,
        )?;
        // Verify that the ibc account root is part of the rollup root
        scroll_verify_zktrie_account_storage_root(
            scroll_header.finalized_proof.finalized_state_root,
            &scroll_client_state.ibc_contract_address,
            &scroll_header.ibc_account_proof.proof,
            &scroll_header.ibc_account_proof.storage_root,
        )?;
        Ok(())
    }
}

pub fn finalized_state_root_key(slot: U256, batch_index: U256) -> H256 {
    sha3::Keccak256::new()
        .chain_update(batch_index.to_big_endian())
        .chain_update(slot.to_big_endian())
        .finalize()
        .into()
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

pub fn scroll_verify_zktrie_storage_proof(
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

pub fn scroll_verify_zktrie_storage_absence(
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
            // https://github.com/scroll-tech/zktrie/blob/9a48ae555bc0454119bf5019e1c9d99a1c25e382/docs/zktrie.md#ethereum-account-leaf-node
            /*
            (The following scheme assumes the big-endian encoding)
            [0:32] (bytes in big-endian)
              [0:16] Reserved with all 0
              [16:24] CodeSize, uint64 in big-endian
              [24:32] Nonce, uint64 in big-endian
            [32:64] Balance
            [64:96] StorageRoot
            [96:128] KeccakCodeHash
            [128:160] PoseidonCodehash
            (total 160 bytes)
            */
            let account =
                TryInto::<[u8; 160]>::try_into(node.data()).map_err(|_| Error::ValueMismatch)?;
            let account_storage_root = &account[64..96];
            if account_storage_root == expected_storage_root.as_ref() {
                Ok(())
            } else {
                Err(Error::ValueMismatch)
            }
        }
    }
}

#[cfg(test)]
mod tests {
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

    use crate::{
        scroll_verify_header, scroll_verify_zktrie_storage_absence,
        scroll_verify_zktrie_storage_proof,
    };

    #[test]
    fn test_scroll_header() {
        let scroll_client_state = ClientState {
            l1_client_id: "blabla".into(),
            chain_id: 0.into(),
            latest_batch_index: 65031,
            frozen_height: Height::default(),
            rollup_contract_address: H160::try_from(
                hex::decode("2d567ece699eabe5afcd141edb7a4f2d0d6ce8a0").unwrap(),
            )
            .unwrap(),
            rollup_finalized_state_roots_slot: 158.into(),
            ibc_contract_address: H160::try_from(
                hex::decode("E52c957533bd932E357046bF721D2Bf2368ef1B7").unwrap(),
            )
            .unwrap(),
            ibc_commitment_slot: 0.into(),
        };
        let scroll_header: Header =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_header.json").unwrap())
                .unwrap();
        let l1_state_root = H256::try_from(
            hex::decode("d36f51bb31957a627d91ca2e9a8f7d8fe0f527293135a4ee177406c78960437d")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            scroll_verify_header(scroll_client_state, scroll_header, l1_state_root),
            Ok(())
        );
    }

    #[test]
    fn test_scroll_l2_contract_slot_exist() {
        let proof: Proof =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_proof.json").unwrap())
                .unwrap();
        assert_eq!(
            scroll_verify_zktrie_storage_proof(
                H256::try_from(
                    hex::decode("1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0")
                        .unwrap()
                )
                .unwrap(),
                proof.key.to_big_endian().into(),
                &proof.value.to_big_endian(),
                &proof.proof
            ),
            Ok(())
        )
    }

    #[test]
    fn test_scroll_l2_contract_slot_absent() {
        let proof: Proof =
            serde_json::from_str(&std::fs::read_to_string("tests/scroll_absent.json").unwrap())
                .unwrap();
        assert_eq!(
            scroll_verify_zktrie_storage_absence(
                H256::try_from(
                    hex::decode("1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0")
                        .unwrap()
                )
                .unwrap(),
                proof.key.to_big_endian().into(),
                &proof.proof
            ),
            Ok(())
        )
    }
}
