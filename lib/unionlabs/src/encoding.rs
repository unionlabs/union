use core::fmt::Debug;

pub trait Encoding: Send + Sync {}

#[cfg(feature = "ethabi")]
pub enum EthAbi {}
#[cfg(feature = "ethabi")]
impl Encoding for EthAbi {}

#[cfg(feature = "proto")]
pub enum Proto {}
#[cfg(feature = "proto")]
impl Encoding for Proto {}

// TODO: Feature gate this
#[cfg(feature = "ssz")]
pub enum Ssz {}
#[cfg(feature = "ssz")]
impl Encoding for Ssz {}

#[cfg(feature = "json")]
pub enum Json {}
#[cfg(feature = "json")]
impl Encoding for Json {}

#[cfg(feature = "json")]
impl<T> Encode<Json> for T
where
    T: serde::Serialize,
{
    fn encode(self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("json serialization should be infallible")
    }
}

#[cfg(feature = "json")]
impl<T> Decode<Json> for T
where
    T: serde::de::DeserializeOwned,
{
    type Error = serde_json::Error;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(bytes)
    }
}

#[cfg(feature = "json")]
static_assertions::assert_impl_all!(u8: Encode<Json>);
#[cfg(feature = "json")]
static_assertions::assert_impl_all!(&u8: Encode<Json>);

#[cfg(feature = "bcs")]
pub enum Bcs {}
#[cfg(feature = "bcs")]
impl Encoding for Bcs {}

#[cfg(feature = "bcs")]
impl<T> Encode<Bcs> for T
where
    T: serde::Serialize,
{
    fn encode(self) -> Vec<u8> {
        bcs::to_bytes(&self).expect("json serialization should be infallible")
    }
}

#[cfg(feature = "bcs")]
impl<T> Decode<Bcs> for T
where
    T: serde::de::DeserializeOwned,
{
    type Error = bcs::Error;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        bcs::from_bytes(bytes)
    }
}

#[cfg(feature = "bcs")]
static_assertions::assert_impl_all!(u8: Encode<Bcs>);
#[cfg(feature = "bcs")]
static_assertions::assert_impl_all!(&u8: Encode<Bcs>);

pub trait Encode<Enc: Encoding>: Sized {
    fn encode(self) -> Vec<u8>;
}

pub trait Decode<Enc: Encoding>: Sized {
    // if you have non-thread-safe errors wtf are you doing
    type Error: Debug + Send + Sync;

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
