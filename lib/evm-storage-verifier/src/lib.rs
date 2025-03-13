use error::Error;
use hash_db::HashDB;
use memory_db::{HashKey, MemoryDB};
use rlp::RlpDecodable;
use trie_db::{Trie, TrieDBBuilder};
use unionlabs::{
    ensure,
    primitives::{H160, H256, U256},
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

    let trie = TrieDBBuilder::<EthLayout>::new(&db, &root).build();
    Ok(trie.get(&keccak_256(key.as_ref()))?)
}

#[cfg(test)]
mod tests {
    use alloy::rpc::types::EIP1186AccountProofResponse;
    use hex_literal::hex;
    use lazy_static::lazy_static;

    use super::*;

    lazy_static! {
        pub static ref VALID_ABSENCE_PROOF: EIP1186AccountProofResponse =
            serde_json::from_str(include_str!("./test/valid_absence_proof_sepolia.json")).unwrap();
        pub static ref VALID_STORAGE_PROOF: EIP1186AccountProofResponse =
            serde_json::from_str(include_str!("./test/valid_storage_proof_sepolia.json")).unwrap();
    }

    // Fetch a finality update to obtain a state root at a certain height, then fetch the proofs based on that height
    const STATE_ROOT_AT_7882953: H256 = H256::new(hex!(
        "545e7cf676baca0fad067f9884fbb2a42090c0fa63a00c217c60688917deee6e"
    ));

    #[test]
    fn verify_storage_absence_works() {
        assert_eq!(
            verify_storage_absence(
                H256::new(VALID_ABSENCE_PROOF.storage_hash.0),
                U256::from_be_bytes(VALID_ABSENCE_PROOF.storage_proof[0].key.as_b256().0),
                &VALID_ABSENCE_PROOF.storage_proof[0].proof,
            ),
            Ok(true)
        );
    }

    #[test]
    fn verify_storage_absence_fails_when_altered_proof() {
        let proof = VALID_ABSENCE_PROOF.storage_proof[0]
            .proof
            .iter()
            .cloned()
            .map(|x| [x, vec![1].into()].concat());
        assert!(matches!(
            verify_storage_absence(
                H256::new(VALID_ABSENCE_PROOF.storage_hash.0),
                U256::from_be_bytes(VALID_ABSENCE_PROOF.storage_proof[0].key.as_b256().0),
                proof,
            ),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_storage_absence_fails_when_altered_root() {
        let mut storage_hash = VALID_ABSENCE_PROOF.storage_hash.0;
        storage_hash[0] ^= 0xFF;

        assert!(matches!(
            verify_storage_absence(
                H256::new(storage_hash),
                U256::from_be_bytes(VALID_ABSENCE_PROOF.storage_proof[0].key.as_b256().0),
                &VALID_ABSENCE_PROOF.storage_proof[0].proof,
            ),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_storage_absence_fails_when_altered_key() {
        let mut key = VALID_ABSENCE_PROOF.storage_proof[0].key.as_b256().0;
        key[0] ^= 0xFF;

        assert!(matches!(
            verify_storage_absence(
                H256::new(VALID_ABSENCE_PROOF.storage_hash.0),
                U256::from_be_bytes(key),
                &VALID_ABSENCE_PROOF.storage_proof[0].proof,
            ),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_storage_verification_fails_when_absent() {
        assert!(matches!(
            verify_storage_proof(
                H256::new(VALID_ABSENCE_PROOF.storage_hash.0),
                U256::from_be_bytes(VALID_ABSENCE_PROOF.storage_proof[0].key.as_b256().0),
                &VALID_ABSENCE_PROOF.storage_proof[0]
                    .value
                    .to_be_bytes::<32>(),
                &VALID_ABSENCE_PROOF.storage_proof[0].proof,
            ),
            Err(Error::ValueMissing { .. })
        ));
    }

    #[test]
    fn verify_storage_verification_works() {
        assert_eq!(
            verify_storage_proof(
                H256::new(VALID_STORAGE_PROOF.storage_hash.0),
                U256::from_be_bytes(VALID_STORAGE_PROOF.storage_proof[0].key.as_b256().0),
                &rlp::encode(
                    &VALID_STORAGE_PROOF.storage_proof[0]
                        .value
                        .to_be_bytes::<32>()
                        .as_slice()
                ),
                &VALID_STORAGE_PROOF.storage_proof[0].proof,
            ),
            Ok(())
        );
    }

    #[test]
    fn verify_absence_verification_fails_when_key_exists() {
        assert_eq!(
            verify_storage_absence(
                H256::new(VALID_STORAGE_PROOF.storage_hash.0),
                U256::from_be_bytes(VALID_STORAGE_PROOF.storage_proof[0].key.as_b256().0),
                &VALID_STORAGE_PROOF.storage_proof[0].proof,
            ),
            Ok(false)
        );
    }

    #[test]
    fn verify_storage_verification_fails_when_altered_root() {
        let mut storage_hash = VALID_STORAGE_PROOF.storage_hash.0;
        storage_hash[0] ^= 0xFF;

        assert!(matches!(
            verify_storage_proof(
                H256::new(storage_hash),
                U256::from_be_bytes(VALID_STORAGE_PROOF.storage_proof[0].key.as_b256().0),
                &rlp::encode(
                    &VALID_STORAGE_PROOF.storage_proof[0]
                        .value
                        .to_be_bytes::<32>()
                        .as_slice()
                ),
                &VALID_STORAGE_PROOF.storage_proof[0].proof,
            ),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_storage_verification_fails_when_altered_proof() {
        let proof = VALID_ABSENCE_PROOF.storage_proof[0]
            .proof
            .iter()
            .cloned()
            .map(|x| [x, vec![1].into()].concat());

        assert!(matches!(
            verify_storage_proof(
                H256::new(VALID_STORAGE_PROOF.storage_hash.0),
                U256::from_be_bytes(VALID_STORAGE_PROOF.storage_proof[0].key.as_b256().0),
                &rlp::encode(
                    &VALID_STORAGE_PROOF.storage_proof[0]
                        .value
                        .to_be_bytes::<32>()
                        .as_slice()
                ),
                proof,
            ),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_storage_verification_fails_when_altered_key() {
        let mut key = VALID_STORAGE_PROOF.storage_proof[0].key.as_b256().0;
        key[0] ^= 0xFF;

        assert!(matches!(
            verify_storage_proof(
                H256::new(VALID_STORAGE_PROOF.storage_hash.0),
                U256::from_be_bytes(key),
                &rlp::encode(
                    &VALID_STORAGE_PROOF.storage_proof[0]
                        .value
                        .to_be_bytes::<32>()
                        .as_slice()
                ),
                &VALID_STORAGE_PROOF.storage_proof[0].proof,
            ),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_storage_verification_fails_when_altered_value() {
        let mut value = VALID_STORAGE_PROOF.storage_proof[0]
            .value
            .to_be_bytes::<32>();
        value[0] ^= 0xFF;

        assert!(matches!(
            verify_storage_proof(
                H256::new(VALID_STORAGE_PROOF.storage_hash.0),
                U256::from_be_bytes(VALID_STORAGE_PROOF.storage_proof[0].key.as_b256().0),
                &rlp::encode(&value.as_slice()),
                &VALID_STORAGE_PROOF.storage_proof[0].proof,
            ),
            Err(Error::ValueMismatch { .. })
        ));
    }

    #[test]
    fn verify_account_proof_works() {
        assert_eq!(
            verify_account_storage_root(
                STATE_ROOT_AT_7882953,
                &H160::new(*VALID_STORAGE_PROOF.address.as_ref()),
                &VALID_STORAGE_PROOF.account_proof,
                &H256::new(VALID_STORAGE_PROOF.storage_hash.0),
            ),
            Ok(())
        );
    }

    #[test]
    fn verify_account_proof_fails_with_altered_root() {
        let mut root = STATE_ROOT_AT_7882953;
        root[0] ^= 0xFF;
        assert!(matches!(
            verify_account_storage_root(
                root,
                &H160::new(*VALID_STORAGE_PROOF.address.as_ref()),
                &VALID_STORAGE_PROOF.account_proof,
                &H256::new(VALID_STORAGE_PROOF.storage_hash.0),
            ),
            Err(Error::Trie(_))
        ));
    }

    #[test]
    fn verify_account_proof_fails_with_altered_storage_root() {
        let mut storage_root = VALID_STORAGE_PROOF.storage_hash.0;
        storage_root[0] ^= 0xFF;
        assert!(matches!(
            verify_account_storage_root(
                STATE_ROOT_AT_7882953,
                &H160::new(*VALID_STORAGE_PROOF.address.as_ref()),
                &VALID_STORAGE_PROOF.account_proof,
                &H256::new(storage_root),
            ),
            Err(Error::ValueMismatch { .. })
        ));
    }
}
