use crate::errors::Error;
use crate::internal_prelude::*;
use ethereum_consensus::types::{Address, H256, U64};
use patricia_merkle_trie::{
    keccak::{keccak_256, KeccakHasher},
    EIP1186Layout, StorageProof,
};
use trie_db::{Trie, TrieDBBuilder};

/// ExecutionVerifier is a verifier of execution layer's state
/// The proof spec follows EIP-1186: https://eips.ethereum.org/EIPS/eip-1186
#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionVerifier;

impl ExecutionVerifier {
    /// get a value corresponding to `key` and `proof`
    pub fn verify(
        &self,
        root: H256,
        key: &[u8],
        proof: Vec<Vec<u8>>,
    ) -> Result<Option<Vec<u8>>, Error> {
        let db = StorageProof::new(proof).into_memory_db::<KeccakHasher>();
        let root: primitive_types::H256 = root.into();
        let trie = TrieDBBuilder::<EIP1186Layout<KeccakHasher>>::new(&db, &root).build();
        Ok(trie.get(&keccak_256(key))?)
    }

    /// check if a value corresponding to `key` exists
    pub fn verify_membership(
        &self,
        root: H256,
        key: &[u8],
        expected_value: &[u8],
        proof: Vec<Vec<u8>>,
    ) -> Result<(), Error> {
        if let Some(value) = self.verify(root, key, proof)? {
            if value == expected_value {
                Ok(())
            } else {
                Err(Error::ExecutionValueMismatch(value, expected_value.into()))
            }
        } else {
            Err(Error::ExecutionValueNonExist)
        }
    }

    /// check if a value corresponding to `key` doesn't exist
    pub fn verify_non_membership(
        &self,
        root: H256,
        key: &[u8],
        proof: Vec<Vec<u8>>,
    ) -> Result<(), Error> {
        if let Some(_) = self.verify(root, key, proof)? {
            Err(Error::ExecutionValueExist)
        } else {
            Ok(())
        }
    }

    /// verify an account with a given address and proof
    pub fn verify_account(
        &self,
        root: H256,
        address: &Address,
        proof: Vec<Vec<u8>>,
    ) -> Result<Option<Account>, Error> {
        if let Some(value) = self.verify(root, &address.0.as_slice(), proof)? {
            Ok(Some(Account::from_rlp_bytes(&value)?))
        } else {
            Ok(None)
        }
    }

    /// check if an account's storage root matches `storage_root`
    pub fn verify_account_storage_root(
        &self,
        root: H256,
        address: &Address,
        proof: Vec<Vec<u8>>,
        storage_root: H256,
    ) -> Result<bool, Error> {
        if let Some(account) = self.verify_account(root, address, proof)? {
            Ok(account.storage_root == storage_root)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Account {
    pub nonce: U64,
    pub balance: Vec<u8>,
    pub storage_root: H256,
    pub code_hash: H256,
}

impl Account {
    pub fn from_rlp_bytes(bz: &[u8]) -> Result<Account, Error> {
        let r = rlp::Rlp::new(bz);
        Ok(Account {
            nonce: r.val_at::<u64>(0)?.into(),
            balance: r.val_at::<Vec<u8>>(1)?,
            storage_root: H256::from_slice(r.val_at::<Vec<u8>>(2)?.as_slice()),
            code_hash: H256::from_slice(r.val_at::<Vec<u8>>(3)?.as_slice()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Account, ExecutionVerifier};
    use ethereum_consensus::types::{Address, H256, U64};
    use hex_literal::hex;

    #[test]
    fn test_account_deserialization() {
        let acc = Account::from_rlp_bytes(
            hex!("F8440180A02988BB89D212527A6054FEC481672B5CDD01BDF7287129442E82BB7569A412F9A0CF76E7C6FA61CCA89FEE643691266BB1F2721C2D2EEB3063A5E545560ABC2B7A").as_slice()
        );
        assert!(acc.is_ok());
        let acc = acc.unwrap();
        assert_eq!(acc.nonce, U64(1));
        assert_eq!(acc.balance, Vec::<u8>::new());
        assert_eq!(
            acc.storage_root.0,
            hex!("2988BB89D212527A6054FEC481672B5CDD01BDF7287129442E82BB7569A412F9")
        );
        assert_eq!(
            acc.code_hash.0,
            hex!("CF76E7C6FA61CCA89FEE643691266BB1F2721C2D2EEB3063A5E545560ABC2B7A")
        )
    }

    #[test]
    fn test_account_verification() {
        let proof = vec![
            hex!("f901d180a09199d4ddc4f618c0df40c0e1e09eaf2394cd21d566d841b654f3f268196922d0a0bac36050a7d1931b8d6f027075410a85587c649f2d0b30e8ffe967cb3329314ca03919f1f0815704a954616d26504c9201132454a1c0023252294c1abbe0fab26fa0e72e174077c047357c47cba596110765043277d24c55c787ecb164e33a7f1aa5a0de86ea5531307567132648d5c7956cb6082d6803f3dbc9e16b2dd20b320ca93aa0c2c799b60a0cd6acd42c1015512872e86c186bcf196e85061e76842f3b7cf860a088126df40baa53d4d60c0e2a004b6ee8506f131573c750649380e74662093855a02e0d86c3befd177f574a20ac63804532889077e955320c9361cd10b7cc6f5809a0c326f61dd1e74e037d4db73aede5642260bf92869081753bbace550a73989aeda069d63e492e4c3aa54393df9bc12809c9bfc6482b3feb16f2877d7a3e6857d94780a029087b3ba8c5129e161e2cb956640f4d8e31a35f3f133c19a1044993def98b61a08d65cbe14c995d8fe7c7343e9aa31efc7dd81acb0ee940ee565613d8bbbbaa02a0bb12ddf18cf418b9bb5164d2c0caad9e4a29bdca8f1a0c9ed16dd8095f8792fba0144540d36e30b250d25bd5c34d819538742dc54c2017c4eb1fabb8e45f72759180").to_vec(),
            hex!("f8518080a0b595706019b55ae9c4784db71e12bc68d3c991fc1277327e8d63014d10137f7b8080808080808080a04e41195493413c0bbe1fd524bbac490ed81e002fbf4d3d769e0be3452466de0c8080808080").to_vec(),
            hex!("f869a020fff6b964c3925a3b7475bdd2ad96660593de57a6a55a3ef0c82303af814889b846f8440180a02988bb89d212527a6054fec481672b5cdd01bdf7287129442e82bb7569a412f9a0cf76e7c6fa61cca89fee643691266bb1f2721c2d2eeb3063a5e545560abc2b7a").to_vec(),
        ];
        let root =
            H256::from_hex("6a3c41347943fdeab40fb6f0cff088bc81032c86a22b69c67c83b79b72cbb0b4")
                .unwrap();
        let address = Address(hex!("12496c9aa0e6754c897ca88c1d53fea9b19b8aff"));

        let verifier = ExecutionVerifier::default();
        let res = verifier.verify_account(root.clone(), &address, proof.clone());
        assert!(res.is_ok());
        let acc = res.unwrap();
        assert!(acc.is_some());
        assert_eq!(
            acc.clone().unwrap().storage_root.0,
            hex!("2988BB89D212527A6054FEC481672B5CDD01BDF7287129442E82BB7569A412F9")
        );

        let res =
            verifier.verify_account_storage_root(root, &address, proof, acc.unwrap().storage_root);
        assert!(res.is_ok());
        assert!(res.unwrap());
    }
}
