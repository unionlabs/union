//! This library exposes generic encoding and decoding traits, to allow for abstracting over the
//! encoding used for a type without needing to worry about the specific implementation details of
//! said encoding.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::pedantic, missing_docs)]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Debug;

/// Encode a value with the given [`Encoding`].
pub trait Encode<E: Encoding>: Sized {
    /// Encode `self` to bytes.
    ///
    /// Implementations must ensure that this method does ***not*** panic. Encoding is assumed to be
    /// *infallible* for well-formed types. Different encodings may have different definitions of
    /// well-formed (for example, `serde_json` requires map keys to be strings). Consumers of this
    /// trait must ensure they uphold any invariants required by the encoding scheme being used.
    fn encode(self) -> Vec<u8>;
}

/// Decode a value using the given [`Encoding`].
pub trait Decode<E: Encoding>: Sized {
    /// Possible errors that may occur during decoding.
    ///
    /// Note that this is `Send` + `Sync`. If the error type for the encoding scheme is not
    /// thread-safe, reconsider your life choices.
    type Error: Debug + Send + Sync;

    /// Attempt to decode a value of `Self` from bytes.
    ///
    /// # Errors
    ///
    /// This method should return an error if the decoding fails.
    ///
    /// # Panics
    ///
    /// Implementations must ensure that this method does ***not*** panic. If an error is
    /// encountered while decoding, it must be returned as an `Err`. Consumers of this trait can
    /// (and should) rely on a correct implementation.
    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>;
}

/// Convenience type for naming the decoding error of a type, given an encoding:
///
/// ```rust
/// # use unionlabs_encoding::{Decode, DecodeErrorOf, Encoding};
/// fn decode<T: Decode<E>, E: Encoding>(bytes: &[u8]) -> Result<T, DecodeErrorOf<E, T>> {
///     <T as Decode<E>>::decode(bytes)
/// }
/// ```
pub type DecodeErrorOf<E, T> = <T as Decode<E>>::Error;

/// A convenience trait for [`Encode`] that allows specifying the [`Encoding`] at the callsite,
/// rather than requiring fully qualified syntax.
///
///
/// ```rust
/// # use unionlabs_encoding::{Encode, EncodeAs, Encoding};
/// fn encode_value<T, E1, E2>(t: T) -> (Vec<u8>, Vec<u8>)
/// where
///     T: Clone + Encode<E1> + Encode<E2>,
///     E1: Encoding,
///     E2: Encoding,
/// {
///     let e1_bytes = t.clone().encode_as::<E1>();
///     let e2_bytes = t.encode_as::<E2>();
///
///     (e1_bytes, e2_bytes)
/// }
/// ```
pub trait EncodeAs {
    /// Encode `self` with the given encoding.
    fn encode_as<E: Encoding>(self) -> Vec<u8>
    where
        Self: Encode<E>,
    {
        Encode::<E>::encode(self)
    }
}

impl<T> EncodeAs for T {}

/// A convenience trait for [`Decode`] that allows specifying the [`Encoding`] at the callsite,
/// rather than  requiring fully qualified syntax.
///
///
/// ```rust
/// # use unionlabs_encoding::{Decode, DecodeAs, Encoding};
/// fn decode_value<T, E1, E2>(e1_bytes: &[u8], e2_bytes: &[u8]) -> (T, T)
/// where
///     T: Decode<E1> + Decode<E2>,
///     E1: Encoding,
///     E2: Encoding,
/// {
///     let e1_t = T::decode_as::<E1>(e1_bytes).unwrap();
///     let e2_t = T::decode_as::<E1>(e2_bytes).unwrap();
///
///     (e1_t, e2_t)
/// }
/// ```
pub trait DecodeAs {
    /// Decode `self` using the given encoding.
    ///
    /// # Errors
    ///
    /// This function will return an error if [`<T as Decode<E>>::decode()`](Decode::decode) fails.
    fn decode_as<E: Encoding>(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Decode<E>,
    {
        Decode::<E>::decode(bytes)
    }
}

impl<T> DecodeAs for T {}

/// An encoding scheme.
///
/// Encodings represent a way to *infallibly* encode a given value to bytes, and *fallibly* decode a
/// value from bytes.
pub trait Encoding: Send + Sync {}

/// [SSZ](https://github.com/ethereum/consensus-specs/blob/dev/ssz/simple-serialize.md) encoding.
///
/// This encoding scheme is implemented with the [`ssz`] crate.
///
/// # Encoding invariants
///
/// There are no specific encoding invariants for SSZ encoding. Any object that has a correct (i.e.
/// non-panicking) [`ssz::Ssz`] implementation can be successfully encoded. The derive macro from
/// the [`ssz`] crate can provide such an implementation.
#[cfg(feature = "ssz")]
pub enum Ssz {}
#[cfg(feature = "ssz")]
impl Encoding for Ssz {}

#[cfg(feature = "ssz")]
impl<T> Encode<Ssz> for T
where
    T: ssz::Ssz,
{
    fn encode(self) -> Vec<u8> {
        ssz::Ssz::as_ssz_bytes(&self)
    }
}

#[cfg(feature = "ssz")]
impl<T> Decode<Ssz> for T
where
    T: ssz::Ssz,
{
    type Error = ssz::decode::DecodeError;

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        ssz::Ssz::from_ssz_bytes(bytes)
    }
}

/// [JSON](https://www.json.org/) encoding.
///
/// This encoding scheme is implemented with the [`serde_json`] crate.
///
/// # Encoding invariants
///
/// `serde_json` requires that all map keys are string-like (i.e. serialize via one of serializer's
/// string methods). Attempting to encode a map with non-string keys will panic.
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

/// [Bincode](https://github.com/bincode-org/bincode) encoding.
///
/// This encoding scheme is implemented with the [`bincode`] crate, using the
/// [`legacy`](bincode::config::legacy) encoding.
///
/// # Encoding invariants
///
/// There are no specific encoding invariants for bincode encoding. Any object that has a correct
/// (i.e. non-panicking) [`bincode::Encode`] implementation can be successfully encoded. The derive
/// macros from the [`bincode`] crate can provide such an implementation.
#[cfg(feature = "bincode")]
pub enum Bincode {}
#[cfg(feature = "bincode")]
impl Encoding for Bincode {}

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

/// [EthAbi](https://docs.soliditylang.org/en/latest/abi-spec.html) params encoding.
///
/// This encoding scheme is implemented indirectly via the [`alloy_sol_types`] crate. Since this
/// crate does not provide derive macros for it's various encoding and decoding traits, and instead
/// requires one to use the `sol!` macro, a blanket implementation is not provided. Instead, there
/// is a helper macro [`impl_ethabi_via_try_from_into`] that can be used to implement
/// `Encode<EthAbi>`/`Decode<EthAbi>` for your domain type via a conversion to/from the generated
/// `sol!` type:
///
/// ```rust
/// # use unionlabs_encoding::{impl_ethabi_via_try_from_into};
/// # use alloy_sol_types::sol;
/// # use core::num::{NonZeroU32, TryFromIntError};
/// pub struct Struct {
///     field: NonZeroU32
/// }
///
/// sol! {
///     struct SolStruct {
///         uint32 field;
///     }
/// }
///
/// impl_ethabi_via_try_from_into!(Struct => SolStruct);
///
/// impl From<Struct> for SolStruct {
///     fn from(value: Struct) -> Self {
///         Self {
///             field: value.field.get(),
///         }
///     }
/// }
///
/// impl TryFrom<SolStruct> for Struct {
///     type Error = TryFromIntError;
///
///     fn try_from(value: SolStruct) -> Result<Self, Self::Error> {
///         Ok(Self {
///             field: value.field.try_into()?,
///         })
///     }
/// }
/// ```
///
/// # Encoding invariants
///
/// There are no specific encoding invariants for ethabi encoding. Any object that has a correct
/// (i.e. non-panicking) [`alloy_sol_types::SolValue::abi_decode_params`] and `Into` implementation
/// can be successfully encoded. The [`alloy_sol_types::sol!`] macro can provide the former, however
/// the latter must be implemented correctly by the implementor.
///
/// ## Params ("un-wrapped") encoding
///
/// This encoding is expected to follow the params encoding as defined [here](https://docs.rs/alloy/latest/alloy/dyn_abi/abi/index.html#encodedecode_params). This ensures that tuples (i.e structs) are not length prefixed when encoded.
#[cfg(feature = "ethabi")]
pub enum EthAbi {}
#[cfg(feature = "ethabi")]
impl Encoding for EthAbi {}

/// Error type used by the [`Decode`] implementation generated by [`impl_ethabi_via_try_from_into`].
#[cfg(feature = "ethabi")]
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[expect(missing_docs, reason = "#[error] attributes provide documentation")]
pub enum TryFromEthAbiBytesError<E> {
    #[error("unable to convert from the raw prost type")]
    Convert(#[source] E),
    #[error("unable to decode from raw ethabi bytes")]
    Decode(#[from] alloy_sol_types::Error),
}

#[cfg(feature = "ethabi")]
#[doc(hidden)]
pub use alloy_sol_types;

/// Implement [`EthAbi`] encoding via conversion to/from a [`sol!`](alloy_sol_types::sol) generated
/// type. See the documentation on [`EthAbi`] for more information.
#[cfg(feature = "ethabi")]
#[macro_export]
macro_rules! impl_ethabi_via_try_from_into {
    ($T:path => $EthAbi:path) => {
        impl $crate::Decode<$crate::EthAbi> for $T {
            type Error = $crate::TryFromEthAbiBytesError<<$T as TryFrom<$EthAbi>>::Error>;

            fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
                <$EthAbi as $crate::alloy_sol_types::SolValue>::abi_decode_params(bytes, false)
                    .map_err($crate::TryFromEthAbiBytesError::Decode)
                    .and_then(|abi| {
                        abi.try_into()
                            .map_err($crate::TryFromEthAbiBytesError::Convert)
                    })
            }
        }

        impl $crate::Encode<$crate::EthAbi> for $T {
            fn encode(self) -> Vec<u8> {
                <$EthAbi as $crate::alloy_sol_types::SolValue>::abi_encode_params(
                    &Into::<$EthAbi>::into(self),
                )
            }
        }
    };
}

/// [Protobuf](https://protobuf.dev) encoding.
///
/// This encoding scheme is implemented indirectly via the [`prost`] crate. Due to the nature of
/// protobuf schema files being separate from source code, and the `prost` generated code often
/// lacking in expressiveness, a blanket implementation is not provided. Instead, there is a helper
/// macro [`impl_proto_via_try_from_into`] that can be used to implement
/// `Encode<Proto>`/`Decode<Proto>` for your domain type via a conversion to/from the generated
/// `prost` type:
///
/// ```rust
/// # use unionlabs_encoding::impl_proto_via_try_from_into;
/// # use core::num::{NonZeroU32, TryFromIntError};
/// pub struct Struct {
///     field: NonZeroU32
/// }
///
/// // generated with `prost`
/// #[derive(Clone, PartialEq, ::prost::Message)]
/// pub struct ProtoStruct {
///     #[prost(uint32, tag = "1")]
///     pub field: u32,
/// }
/// impl ::prost::Name for ProtoStruct {
///     const NAME: &'static str = "Struct";
///     const PACKAGE: &'static str = "package";
///     fn full_name() -> ::prost::alloc::string::String {
///         ::prost::alloc::format!("package.{}", Self::NAME)
///     }
/// }
///
/// impl_proto_via_try_from_into!(Struct => ProtoStruct);
///
/// impl From<Struct> for ProtoStruct {
///     fn from(value: Struct) -> Self {
///         Self {
///             field: value.field.get(),
///         }
///     }
/// }
///
/// impl TryFrom<ProtoStruct> for Struct {
///     type Error = TryFromIntError;
///
///     fn try_from(value: ProtoStruct) -> Result<Self, Self::Error> {
///         Ok(Self {
///             field: value.field.try_into()?,
///         })
///     }
/// }
/// ```
///
/// # Encoding invariants
///
/// There are no specific encoding invariants for protobuf encoding. Any object that has a correct
/// (i.e. non-panicking) [`prost::Message::encode_to_vec`] and `Into` implementation can be
/// successfully encoded. [`prost`] code generation can provide the former, however the latter must
/// be implemented correctly by the implementor.
///
/// ## Standard (non-length-delimited) encoding
///
/// This encoding is expected to be standard protobuf encoding. If length-delimited encoding is
/// required, use `prost` directly.
///
/// # Protobuf type urls
///
/// This crate also exports [`TypeUrl`], which is analogous to [`prost::Name`].
/// [`impl_proto_via_try_from_into`] will implement this for your type automatically via the
/// provided `prost` type.
#[cfg(feature = "proto")]
pub enum Proto {}
#[cfg(feature = "proto")]
impl Encoding for Proto {}

#[cfg(feature = "proto")]
#[doc(hidden)]
pub use prost;

/// A type that has a protobuf type url.
///
/// See the documentation for protobuf [Any](https://protobuf.dev/reference/protobuf/google.protobuf/#any) for more information.
#[cfg(feature = "proto")]
pub trait TypeUrl {
    /// Get the type url for this type.
    fn type_url() -> alloc::string::String;
}

/// Error type used by the [`Decode`] implementation generated by [`impl_proto_via_try_from_into`].
#[cfg(feature = "proto")]
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[expect(missing_docs, reason = "#[error] attributes provide documentation")]
pub enum TryFromProtoBytesError<E> {
    #[error("unable to convert from the raw prost type")]
    TryFromProto(#[source] E),
    #[error("unable to decode from raw proto bytes")]
    Decode(#[source] prost::DecodeError),
}

/// Implement [`Proto`] encoding via conversion to/from a [`prost`] generated type. See the
/// documentation on [`Proto`] for more information.
#[cfg(feature = "proto")]
#[macro_export]
macro_rules! impl_proto_via_try_from_into {
    ($({ for($($P:ident)+) $(where encode($($encode:tt)*) decode($($decode:tt)*))? })? $T:ty => $Proto:ty) => {
        impl $(<$($P)+>)?
            $crate::Decode<$crate::Proto>
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

        impl $(<$($P)+>)? $crate::Encode<$crate::Proto> for $T $($(where $($encode)*)?)? {
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

/// [BCS](bcs) encoding.
///
/// This encoding scheme is implemented with the [`bcs`] crate.
///
/// # Encoding invariants
///
/// Due to `bcs` being a non-self-describing format, certain issues may be encountered if using particularly complicated serde representations. Consult the [serde issue tracker](https://github.com/serde-rs/serde/issues) if there is any unexpected behaviour.
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
