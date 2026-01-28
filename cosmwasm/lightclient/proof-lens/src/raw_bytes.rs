use std::convert::Infallible;

use unionlabs::{
    encoding::{Bincode, Decode, Encode},
    primitives::Bytes,
};

#[derive(Debug)]
pub struct RawBytes(pub Bytes);

impl Encode<Bincode> for RawBytes {
    fn encode(self) -> Vec<u8> {
        self.0.into()
    }
}

impl Decode<Bincode> for RawBytes {
    type Error = Infallible;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(bytes.into()))
    }
}
