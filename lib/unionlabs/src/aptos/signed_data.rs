use ed25519_zebra::SigningKey;
use macros::model;
use sha2::Digest;
use unionlabs_primitives::H512;

use crate::encoding::{Bincode, Encode};

#[model]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SignedData<T> {
    pub signature: H512,
    pub data: T,
}

impl<T: Encode<Bincode> + Clone> SignedData<T> {
    pub fn sign(signing_key: &SigningKey, data: T) -> Self {
        let signature = signing_key.sign(&Self::internal_hash(&data));
        Self {
            signature: H512::new(signature.to_bytes()),
            data,
        }
    }

    fn internal_hash(data: &T) -> Vec<u8> {
        sha2::Sha256::new()
            .chain_update(data.clone().encode())
            .finalize()
            .to_vec()
    }

    pub fn hash(&self) -> Vec<u8> {
        Self::internal_hash(&self.data)
    }
}
