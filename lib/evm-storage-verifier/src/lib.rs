use error::Error;
use hash_db::HashDB;
use memory_db::{HashKey, MemoryDB};
use rlp::RlpDecodable;
use trie_db::{Trie, TrieDBBuilder};
use unionlabs::{
    ensure,
    hash::{H160, H256},
    uint::U256,
};

use crate::rlp_node_codec::{keccak_256, EthLayout, KeccakHasher};

pub mod error;
mod rlp_node_codec;

pub const GENESIS_SLOT: u64 = 0;
pub const GENESIS_EPOCH: u64 = 0;
pub const FAR_FUTURE_EPOCH: u64 = u64::MAX;

pub const BLS_WITHDRAWAL_PREFIX: u8 = 0x00;
pub const ETH1_ADDRESS_WITHDRAWAL_PREFIX: u8 = 0x01;

#[derive(Debug, Clone, RlpDecodable)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: H256,
    pub code_hash: H256,
}

/// Verifies against `root`, if the `expected_value` is stored at `key` by using `proof`.
///
/// * `root`: Storage root of a contract.
/// * `key`: Padded slot number that the `expected_value` should be stored at.
/// * `expected_value`: Expected stored value.
/// * `proof`: Proof that is generated to prove the storage.
///
/// NOTE: You must not trust the `root` unless you verified it by calling [`verify_account_storage_root`].
pub fn verify_storage_proof(
    root: H256,
    key: U256,
    expected_value: &[u8],
    proof: impl IntoIterator<Item = impl AsRef<[u8]>>,
) -> Result<(), Error> {
    match get_node(root, key.to_be_bytes(), proof)? {
        Some(value) if value == expected_value => Ok(()),
        Some(value) => Err(Error::ValueMismatch {
            expected: expected_value.into(),
            actual: value,
        })?,
        None => Err(Error::ValueMissing {
            value: expected_value.into(),
        })?,
    }
}

/// Verifies against `root`, that no value is stored at `key` by using `proof`.
///
/// * `root`: Storage root of a contract.
/// * `key`: Padded slot number that the `expected_value` should be stored at.
/// * `proof`: Proof that is generated to prove the storage.
///
/// NOTE: You must not trust the `root` unless you verified it by calling [`verify_account_storage_root`].
pub fn verify_storage_absence(
    root: H256,
    key: U256,
    proof: impl IntoIterator<Item = impl AsRef<[u8]>>,
) -> Result<bool, Error> {
    Ok(get_node(root, key.to_be_bytes(), proof)?.is_none())
}

/// Verifies if the `storage_root` of a contract can be verified against the state `root`.
///
/// * `root`: Light client update's (attested/finalized) execution block's state root.
/// * `address`: Address of the contract.
/// * `proof`: Proof of storage.
/// * `storage_root`: Storage root of the contract.
///
/// NOTE: You must not trust the `root` unless you verified it by calling [`validate_light_client_update`].
pub fn verify_account_storage_root(
    root: H256,
    address: &H160,
    proof: impl IntoIterator<Item = impl AsRef<[u8]>>,
    storage_root: &H256,
) -> Result<(), Error> {
    match get_node(root, address.as_ref(), proof)? {
        Some(account) => {
            let account = rlp::decode::<Account>(account.as_ref()).map_err(Error::RlpDecode)?;
            ensure(
                &account.storage_root == storage_root,
                Error::ValueMismatch {
                    expected: storage_root.as_ref().into(),
                    actual: account.storage_root.into(),
                },
            )?;
            Ok(())
        }
        None => Err(Error::ValueMissing {
            value: address.as_ref().into(),
        })?,
    }
}

fn get_node(
    root: H256,
    key: impl AsRef<[u8]>,
    proof: impl IntoIterator<Item = impl AsRef<[u8]>>,
) -> Result<Option<Vec<u8>>, Error> {
    let mut db = MemoryDB::<KeccakHasher, HashKey<_>, Vec<u8>>::default();
    proof.into_iter().for_each(|n| {
        db.insert(hash_db::EMPTY_PREFIX, n.as_ref());
    });

    let root: primitive_types::H256 = root.into();
    let trie = TrieDBBuilder::<EthLayout>::new(&db, &root).build();
    Ok(trie.get(&keccak_256(key.as_ref()))?)
}
