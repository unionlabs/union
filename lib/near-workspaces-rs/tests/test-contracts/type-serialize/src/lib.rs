use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::{CurveType, PublicKey};

use std::convert::TryFrom;

#[derive(Default, BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
#[near_bindgen]
struct Contract {}

#[near_bindgen]
impl Contract {
    pub fn pass_pk_back_and_forth(&self, pk: PublicKey) -> PublicKey {
        let mut data = vec![CurveType::ED25519 as u8];
        data.extend(
            bs58::decode("6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp")
                .into_vec()
                .expect("could not convert bs58 to vec"),
        );
        let pk_expected =
            PublicKey::try_from(data).expect("could not create public key from parts");

        assert_eq!(pk, pk_expected);
        pk
    }

    #[result_serializer(borsh)]
    pub fn pass_borsh_pk_back_and_forth(&self, #[serializer(borsh)] pk: PublicKey) -> PublicKey {
        let mut data = vec![CurveType::ED25519 as u8];
        data.extend(
            bs58::decode("6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp")
                .into_vec()
                .expect("could not convert bs58 to vec"),
        );
        let pk_expected =
            PublicKey::try_from(data).expect("could not create public key from parts");

        assert_eq!(pk, pk_expected);
        pk
    }
}
