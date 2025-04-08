use core::fmt::Display;

use thiserror::Error;
use unionlabs::{
    encoding::{Encode, Encoding},
    primitives::{H256, H512},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SignedData<T> {
    data: T,
    signature: H512,
}

#[derive(Debug, Error)]
pub struct SignatureVerificationFailure;

impl Display for SignatureVerificationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "signature verification failure")
    }
}

impl<T: Clone> SignedData<T> {
    pub fn sign<E, V>(data: T, sign: V) -> Self
    where
        E: Encoding,
        for<'a> &'a T: Encode<E>,
        V: Fn(&[u8]) -> H512,
    {
        SignedData {
            signature: sign(&data.encode()),
            data,
        }
    }

    pub fn unwrap_verified<E, V>(
        self,
        pubkey: H256,
        verify_sig: V,
    ) -> Result<T, SignatureVerificationFailure>
    where
        E: Encoding,
        for<'a> &'a T: Encode<E>,
        V: Fn(&[u8], H512, H256) -> Option<bool>,
    {
        if verify_sig(&self.data.encode(), self.signature, pubkey)
            .ok_or(SignatureVerificationFailure)?
        {
            return Ok(self.data);
        }

        Err(SignatureVerificationFailure)
    }
}
