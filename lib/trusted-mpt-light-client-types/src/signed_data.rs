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

pub struct SignatureVerificationFailure;

impl<T: Clone> SignedData<T> {
    pub fn sign<E, V: Fn(&[u8]) -> H512>(data: T, sign: V) -> Self
    where
        E: Encoding,
        T: Encode<E>,
    {
        SignedData {
            signature: sign(&data.clone().encode()),
            data,
        }
    }

    pub fn unwrap_verified<E, V: Fn(&[u8], H512, H256) -> Option<bool>>(
        self,
        pubkey: H256,
        verify_sig: V,
    ) -> Result<T, SignatureVerificationFailure>
    where
        E: Encoding,
        T: Encode<E>,
    {
        if verify_sig(&self.data.clone().encode(), self.signature, pubkey)
            .ok_or(SignatureVerificationFailure)?
        {
            return Ok(self.data);
        }

        Err(SignatureVerificationFailure)
    }
}
