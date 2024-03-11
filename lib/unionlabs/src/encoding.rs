use core::fmt::Debug;

#[cfg(feature = "ethabi")]
use crate::{IntoEthAbi, TryFromEthAbi, TryFromEthAbiBytesError, TryFromEthAbiErrorOf};

pub trait Encoding {}

pub enum EthAbi {}
impl Encoding for EthAbi {}

pub enum Proto {}
impl Encoding for Proto {}

pub trait Encode<Enc: Encoding>: Sized {
    fn encode(self) -> Vec<u8>;
}

pub trait Decode<Enc: Encoding>: Sized {
    type Error: Debug;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>;
}

pub type DecodeErrorOf<Enc, T> = <T as Decode<Enc>>::Error;

#[cfg(feature = "ethabi")]
impl<T: IntoEthAbi> Encode<EthAbi> for T {
    fn encode(self) -> Vec<u8> {
        self.into_eth_abi_bytes()
    }
}

#[cfg(feature = "ethabi")]
impl<T: TryFromEthAbi> Decode<EthAbi> for T
where
    TryFromEthAbiBytesError<TryFromEthAbiErrorOf<T>>: Debug,
{
    type Error = TryFromEthAbiBytesError<TryFromEthAbiErrorOf<T>>;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        T::try_from_eth_abi_bytes(bytes)
    }
}

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
