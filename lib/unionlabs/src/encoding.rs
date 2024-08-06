use core::fmt::Debug;

pub trait Encoding: Send + Sync {}

pub enum EthAbi {}
impl Encoding for EthAbi {}

pub enum Proto {}
impl Encoding for Proto {}

pub enum Ssz {}
impl Encoding for Ssz {}

pub enum Json {}
impl Encoding for Json {}

// TODO: feature gate this
pub enum Borsh {}
impl Encoding for Borsh {}

impl<T> Encode<Json> for T
where
    T: serde::Serialize,
{
    fn encode(self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("json serialization should be infallible")
    }
}

impl<T> Decode<Json> for T
where
    T: serde::de::DeserializeOwned,
{
    type Error = serde_json::Error;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(bytes)
    }
}

static_assertions::assert_impl_all!(u8: Encode<Json>);
static_assertions::assert_impl_all!(&u8: Encode<Json>);

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
