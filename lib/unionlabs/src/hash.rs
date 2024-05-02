use crate::macros::hex_string_array_wrapper;

hex_string_array_wrapper! {
    pub struct H64(pub [u8; 8]);
    pub struct H160(pub [u8; 20]);
    pub struct H256(pub [u8; 32]);
    pub struct H384(pub [u8; 48]);
    pub struct H512(pub [u8; 64]);
    pub struct H2048(pub [u8; 256]);
}

impl H256 {
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        // use this if we ever swap out the inner value for primitive_types::H256
        // self.0.into_iter().flat_map(|n| n.to_le_bytes()).collect()
        self.0.to_vec()
    }
}

impl From<H256> for primitive_types::H256 {
    fn from(value: H256) -> Self {
        Self(value.0)
    }
}

impl From<primitive_types::H256> for H256 {
    fn from(value: primitive_types::H256) -> Self {
        Self(value.0)
    }
}

impl From<H160> for primitive_types::H160 {
    fn from(value: H160) -> Self {
        Self(value.0)
    }
}

impl From<primitive_types::H160> for H160 {
    fn from(value: primitive_types::H160) -> Self {
        Self(value.0)
    }
}
