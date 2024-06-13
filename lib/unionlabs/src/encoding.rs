use core::fmt::Debug;

pub trait Encoding {}

pub enum EthAbi {}
impl Encoding for EthAbi {}

pub enum Proto {}
impl Encoding for Proto {}

pub enum Ssz {}
impl Encoding for Ssz {}

pub enum Binary {}
impl Encoding for Binary {}

pub trait Encode<Enc: Encoding>: Sized {
    fn encode(self) -> Vec<u8>;
}

pub trait Decode<Enc: Encoding>: Sized {
    type Error: Debug;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>;
}

pub type DecodeErrorOf<Enc, T> = <T as Decode<Enc>>::Error;

pub trait EncodeAs {
    fn encode_as<Enc: Encoding>(self) -> Vec<u8>
    where
        Self: Encode<Enc>,
    {
        Encode::<Enc>::encode(self)
    }
}

impl<T> EncodeAs for T {}

pub trait DecodeAs {
    fn decode_as<Enc: Encoding>(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Decode<Enc>,
    {
        Decode::<Enc>::decode(bytes)
    }
}

impl<T> DecodeAs for T {}
