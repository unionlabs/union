use unionlabs::{
    encoding::{Bincode, Encode},
    primitives::{H256, H512},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct SignedData<T: Encode<Bincode>> {
    data: T,
    signature: H512,
}

impl<T: Encode<Bincode> + Clone> SignedData<T> {
    pub fn new(data: T, signature: H512) -> Self {
        SignedData { data, signature }
    }

    pub fn unwrap_verified<V: Fn(&[u8], H512, H256) -> Option<bool>>(
        self,
        pubkey: H256,
        verify_sig: V,
    ) -> Result<T, ()> {
        if verify_sig(&self.data.clone().encode(), self.signature, pubkey).ok_or(())? {
            return Ok(self.data);
        }

        Err(())
    }
}
