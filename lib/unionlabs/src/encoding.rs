use std::fmt::Debug;

#[cfg(feature = "ethabi")]
use crate::{IntoEthAbi, TryFromEthAbi, TryFromEthAbiBytesError, TryFromEthAbiErrorOf};
use crate::{IntoProto, TryFromProto, TryFromProtoBytesError, TryFromProtoErrorOf};

pub trait Encoding {}

pub enum EthAbi {}
impl Encoding for EthAbi {}

pub enum Proto {}
impl Encoding for Proto {}

pub trait Encode<Enc: Encoding> {
    fn encode(self) -> Vec<u8>;
}

pub trait Decode<Enc: Encoding>: Sized {
    type Error: Debug;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>;
}

impl<T: IntoProto> Encode<Proto> for T {
    fn encode(self) -> Vec<u8> {
        self.into_proto_bytes()
    }
}

impl<T: TryFromProto> Decode<Proto> for T
where
    TryFromProtoBytesError<TryFromProtoErrorOf<T>>: Debug,
{
    type Error = TryFromProtoBytesError<TryFromProtoErrorOf<T>>;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        T::try_from_proto_bytes(bytes)
    }
}

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
