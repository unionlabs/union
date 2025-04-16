use core::fmt::Debug;

pub trait Encoding: Send + Sync {}

pub enum EthAbi {}
impl Encoding for EthAbi {}

pub enum Proto {}
impl Encoding for Proto {}

pub enum Ssz {}
impl Encoding for Ssz {}

#[cfg(feature = "bincode")]
pub enum Bincode {}
#[cfg(feature = "bincode")]
impl Encoding for Bincode {}

impl<T> Encode<Ssz> for T
where
    T: ssz::Ssz,
{
    fn encode(self) -> Vec<u8> {
        ssz::Ssz::as_ssz_bytes(&self)
    }
}

impl<T> Decode<Ssz> for T
where
    T: ssz::Ssz,
{
    type Error = ssz::decode::DecodeError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        ssz::Ssz::from_ssz_bytes(bytes)
    }
}

pub enum Json {}
impl Encoding for Json {}

pub enum Bcs {}
impl Encoding for Bcs {}

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

#[cfg(feature = "bincode")]
impl<T> Encode<Bincode> for T
where
    T: bincode::Encode,
{
    fn encode(self) -> Vec<u8> {
        bincode::encode_to_vec(self, bincode::config::legacy())
            .expect("bincode encoding should be infallible")
    }
}

#[cfg(feature = "bincode")]
impl<T> Decode<Bincode> for T
where
    T: bincode::Decode<()>,
{
    type Error = bincode::error::DecodeError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        bincode::decode_from_slice(bytes, bincode::config::legacy()).map(|(t, _)| t)
    }
}

#[cfg(feature = "ethabi")]
#[macro_export]
macro_rules! impl_ethabi_via_try_from_into {
    ($T:path => $EthAbi:path) => {
        impl $crate::encoding::Decode<$crate::encoding::EthAbi> for $T {
            type Error = $crate::TryFromEthAbiBytesErrorAlloy<<$T as TryFrom<$EthAbi>>::Error>;

            fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
                <$EthAbi>::abi_decode_params(bytes, false)
                    .map_err($crate::TryFromEthAbiBytesErrorAlloy::Decode)
                    .and_then(|abi| {
                        abi.try_into()
                            .map_err($crate::TryFromEthAbiBytesErrorAlloy::Convert)
                    })
            }
        }

        impl $crate::encoding::Encode<$crate::encoding::EthAbi> for $T {
            fn encode(self) -> Vec<u8> {
                Into::<$EthAbi>::into(self).abi_encode_params()
            }
        }
    };
}

#[cfg(feature = "proto")]
#[macro_export]
macro_rules! impl_proto_via_try_from_into {
    ($({ for($($P:ident)+) $(where encode($($encode:tt)*) decode($($decode:tt)*))? })? $T:ty => $Proto:ty) => {
        impl $(<$($P)+>)?
            $crate::encoding::Decode<$crate::encoding::Proto>
            for $T $($(where $($decode)*)?)?
        {
            type Error = $crate::TryFromProtoBytesError<<$T as TryFrom<$Proto>>::Error>;

            fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
                <$Proto as $crate::prost::Message>::decode(bytes)
                    .map_err($crate::TryFromProtoBytesError::Decode)
                    .and_then(|proto| {
                        proto
                            .try_into()
                            .map_err($crate::TryFromProtoBytesError::TryFromProto)
                    })
            }
        }

        impl $(<$($P)+>)? $crate::encoding::Encode<$crate::encoding::Proto> for $T $($(where $($encode)*)?)? {
            fn encode(self) -> Vec<u8> {
                $crate::prost::Message::encode_to_vec(&Into::<$Proto>::into(self))
            }
        }

        impl $(<$($P)+>)? $crate::TypeUrl for $T {
            fn type_url() -> String {
                <$Proto as $crate::prost::Name>::type_url()
            }
        }
    };
}

static_assertions::assert_impl_all!(u8: Encode<Json>);
static_assertions::assert_impl_all!(&u8: Encode<Json>);

impl<T> Encode<Bcs> for T
where
    T: serde::Serialize,
{
    fn encode(self) -> Vec<u8> {
        bcs::to_bytes(&self).expect("json serialization should be infallible")
    }
}

impl<T> Decode<Bcs> for T
where
    T: serde::de::DeserializeOwned,
{
    type Error = bcs::Error;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        bcs::from_bytes(bytes)
    }
}

static_assertions::assert_impl_all!(u8: Encode<Bcs>);
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
