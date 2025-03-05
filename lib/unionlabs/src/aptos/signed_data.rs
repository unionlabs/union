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
    pub fn hash(&self) -> Vec<u8> {
        hash_signature_data(self.data.clone())
    }
}

pub fn hash_signature_data<T: Encode<Bincode> + Clone>(data: T) -> Vec<u8> {
    sha2::Sha256::new()
        .chain_update(data.encode())
        .finalize()
        .to_vec()
}
