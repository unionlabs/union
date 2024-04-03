#![doc = include_str!("../README.md")]
#![deny(
    clippy::pedantic,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
#![feature(trait_alias)]

extern crate alloc;

use core::{
    fmt::{Debug, Display},
    ptr::addr_of,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
pub use typenum;

use crate::{
    ibc::core::client::height::{HeightFromStrError, IsHeight},
    id::Bounded,
    validated::Validated,
};

pub const DELAY_PERIOD: u64 = 0;

/// Wrapper types around protos defined in <https://github.com/cosmos/gogoproto/tree/main/protobuf/google/protobuf>, matching the proto module structure.
pub mod google;

pub mod cosmwasm;

/// Defines types that wrap the IBC specification, matching the proto module structure. This also includes `union` extensions to ibc (i.e. types defined in `union.ibc`).
pub mod ibc;

/// Defines types that wrap the tendermint specification, matching the proto module structure.
pub mod tendermint;

/// Defines types that are extended from tendermint in cometbls
pub mod cometbls;

/// Defines types that wrap the cosmos specification, matching the proto module structure.
pub mod cosmos;

/// Various ethereum types. Types that have an IBC counterpart are defined in [`ibc`].
pub mod ethereum;

/// Types specific to the union protocol.
pub mod union;

/// Types specific to the scroll protocol.
pub mod scroll;

/// Wrapper types around [`milagro_bls`] types, providing more conversions and a simpler signing interface.
pub mod bls;

/// Well-known events emitted by ibc-enabled chains.
pub mod events;

pub mod bounded;

pub mod constants;

pub mod proof;

pub mod validated;

pub mod hash;

pub mod encoding;

/// Various identifier types used throughout the IBC stack.
pub mod id;
pub mod signer;
pub mod traits;

// TODO: Replace with something like <https://github.com/recmo/uint>
pub mod uint;

pub(crate) mod macros;

pub mod errors {
    use core::fmt::Debug;

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    #[error("unknown enum variant `{0}`")]
    pub struct UnknownEnumVariant<T>(pub T);

    /// A protobuf field was none unexpectedly.
    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    #[error("missing field `{0}`")]
    pub struct MissingField(pub &'static str);

    /// For fields that are "fake options" from prost, for use in `TryFrom<<Self as Proto>::Proto>`.
    ///
    /// `Self::Error` is expected to have a `MissingField(`[`MissingField`]`)` variant.
    macro_rules! required {
        ($struct_var:ident.$field:ident) => {
            $struct_var
                .$field
                .ok_or(<Self::Error>::MissingField(MissingField(stringify!(
                    $field
                ))))
        };
    }

    // https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files
    pub(crate) use required;

    // Expected one length, but found another.
    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    #[error("invalid length: expected {expected}, found {found}")]
    pub struct InvalidLength {
        // TODO: Make this generic with this enum as individual types
        pub expected: ExpectedLength,
        pub found: usize,
    }

    #[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
    pub enum ExpectedLength {
        #[display(fmt = "exactly {_0}")]
        Exact(usize),
        #[display(fmt = "less than {_0}")]
        LessThan(usize),
        #[display(fmt = "between ({_0}, {_1})")]
        Between(usize, usize),
        #[display(fmt = "greater than or equal to ({_0})")]
        Gte(usize),
    }

    #[derive(Debug, PartialEq, Eq, thiserror::Error)]
    #[error("invalid value: expected {expected}, found {found}")]
    pub struct InvalidValue<T> {
        pub expected: T,
        pub found: T,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TryFromProtoBytesError<E> {
    TryFromProto(E),
    Decode(prost::DecodeError),
}

pub trait TypeUrl {
    fn type_url() -> String;
}

#[cfg(any(feature = "fuzzing", test))]
#[allow(clippy::missing_panics_doc)]
pub mod test_utils {
    use core::{
        fmt::{Debug, Display},
        str::FromStr,
    };

    use crate::encoding::{Decode, Encode, Proto};

    pub fn assert_proto_roundtrip<T>(t: &T)
    where
        T: Encode<Proto> + Decode<Proto> + Debug + Clone + PartialEq,
    {
        let try_from_proto = T::decode(&t.clone().encode()).unwrap();

        assert_eq!(t, &try_from_proto, "proto roundtrip failed");
    }

    pub fn assert_json_roundtrip<T>(t: &T)
    where
        T: serde::Serialize + for<'a> serde::Deserialize<'a> + Debug + PartialEq,
    {
        let from_json = serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap()).unwrap();

        assert_eq!(t, &from_json, "json roundtrip failed");
    }

    pub fn assert_string_roundtrip<T>(t: &T)
    where
        T: Display + FromStr + Debug + PartialEq,
        <T as FromStr>::Err: Debug,
    {
        let from_str = t.to_string().parse::<T>().unwrap();

        assert_eq!(t, &from_str, "string roundtrip failed");
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiBytesError<E> {
    TryFromEthAbi(E),
    Decode(ethers_core::abi::AbiError),
}

/// Due to the broken eth abi rust library, some structures with dynamically
/// sized types are incorrectly encoded (missing a dynamic tuple wrapper)
#[cfg(feature = "ethabi")]
pub struct InlineFields<T>(pub T);

#[cfg(feature = "ethabi")]
impl<T> ethers_core::abi::AbiEncode for InlineFields<T>
where
    T: ethers_core::abi::AbiEncode,
{
    fn encode(self) -> Vec<u8> {
        // Prefixed by the offset at which the 'dynamic' tuple is starting
        ethers_core::abi::AbiEncode::encode(crate::uint::U256::from(32))
            .into_iter()
            .chain(self.0.encode())
            .collect::<Vec<_>>()
    }
}

#[cfg(feature = "ethabi")]
impl<T> ethers_core::abi::AbiDecode for InlineFields<T>
where
    T: ethers_core::abi::AbiDecode,
{
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
        // Wipe the prefix
        Ok(Self(T::decode(
            bytes.as_ref().iter().copied().skip(32).collect::<Vec<_>>(),
        )?))
    }
}

/// An empty string. Will only parse/serialize to/from `""`.
pub type EmptyString<S = String> = Validated<S, EmptyStringValidator>;
pub type EmptyStringValidator = Bounded<0, 0>;

pub use paste::paste;

#[macro_export]
macro_rules! export_wasm_client_type {
    ($type:ident) => {
        const _: $crate::WasmClientType = $crate::WasmClientType::$type;
        $crate::paste! {
            #[no_mangle]
            #[used]
            static [ <WASM_CLIENT_TYPE_ $type> ]: u8 = 0;
        }
    };
}

/// This type is used to discriminate 08-wasm light clients.
/// We need to be able to determine the light client from the light client code itself (not instantiated yet).
/// Light clients supported by voyager must export a `#[no_mangle] static WASM_CLIENT_TYPE: WasmClientType = WasmClientType::...` variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum WasmClientType {
    EthereumMinimal,
    EthereumMainnet,
    Cometbls,
    Tendermint,
    Scroll,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum ClientType {
    Wasm(WasmClientType),
    Tendermint,
    Cometbls,
}

impl FromStr for WasmClientType {
    type Err = WasmClientTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EthereumMinimal" => Ok(WasmClientType::EthereumMinimal),
            "EthereumMainnet" => Ok(WasmClientType::EthereumMainnet),
            "Cometbls" => Ok(WasmClientType::Cometbls),
            "Tendermint" => Ok(WasmClientType::Tendermint),
            "Scroll" => Ok(WasmClientType::Scroll),
            _ => Err(WasmClientTypeParseError::UnknownType(s.to_string())),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WasmClientTypeParseError {
    #[error("unknown wasm client type `{0}`")]
    UnknownType(String),
}

// TODO: Move this and the above types into tools/parse-wasm-client-type, and make it into a library with an optional `parse` feature (so as to not bring in the very heavy wasmparser stack where it's not needed)
pub fn parse_wasm_client_type(
    bz: impl AsRef<[u8]>,
) -> Result<Option<WasmClientType>, WasmClientTypeParseError> {
    wasmparser::Parser::new(0)
        .parse_all(bz.as_ref())
        .find_map(|payload| {
            payload.ok().and_then(|payload| match payload {
                wasmparser::Payload::ExportSection(e) => Some(e),
                _ => None,
            })
        })
        .and_then(|exports| {
            exports.into_iter().find_map(|export| {
                export
                    .ok()
                    .and_then(|export| export.name.strip_prefix("WASM_CLIENT_TYPE_"))
            })
        })
        .map(str::parse)
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "&str",
    into = "String",
    bound(serialize = "", deserialize = "")
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum QueryHeight<H: IsHeight> {
    Latest,
    Specific(H),
}

impl<H: IsHeight> Display for QueryHeight<H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            QueryHeight::Latest => f.write_str("latest"),
            QueryHeight::Specific(height) => f.write_fmt(format_args!("{height}")),
        }
    }
}

impl<H: IsHeight> From<QueryHeight<H>> for String {
    fn from(val: QueryHeight<H>) -> Self {
        val.to_string()
    }
}

impl<H: IsHeight> FromStr for QueryHeight<H> {
    type Err = HeightFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            _ => s.parse().map(Self::Specific),
        }
    }
}

impl<H: IsHeight> TryFrom<&'_ str> for QueryHeight<H> {
    type Error = HeightFromStrError;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

pub trait MaybeRecoverableError: std::error::Error {
    fn is_recoverable(&self) -> bool;
}

fn _is_object_safe(_: &dyn MaybeRecoverableError) {}

#[cfg(not(feature = "arbitrary"))]
pub trait MaybeArbitrary =;
#[cfg(feature = "arbitrary")]
pub trait MaybeArbitrary = for<'a> arbitrary::Arbitrary<'a>;

pub fn impl_maybe_arbitrary<T: MaybeArbitrary>() {}

#[cfg(feature = "arbitrary")]
fn arbitrary_cow_static<T>(
    u: &mut arbitrary::Unstructured,
) -> arbitrary::Result<std::borrow::Cow<'static, T>>
where
    T: ToOwned + ?Sized,
    T::Owned: for<'a> arbitrary::Arbitrary<'a>,
{
    u.arbitrary::<T::Owned>().map(alloc::borrow::Cow::Owned)
}

pub mod never {
    use core::fmt::Display;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
    pub enum Never {}

    impl Display for Never {
        fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match *self {}
        }
    }
}

pub fn ensure<E>(expr: bool, err: E) -> Result<(), E> {
    expr.then_some(()).ok_or(err)
}

pub trait ByteArrayExt<const N: usize> {
    /// Slice into an array at `FROM..(FROM + LEN)`, returning an array of length `LEN`. This will fail to compile if the equivalent slicing would panic at runtime.
    ///
    /// ```compile_fail
    /// # use unionlabs::ByteArrayExt;
    /// let arr = [1, 2, 3, 4, 5];
    ///
    /// // attempt to read `arr[4..(4 + 2)]`
    /// arr.array_slice::<4, 2>();
    /// ```
    ///
    /// ```rust
    /// # use unionlabs::ByteArrayExt;
    /// # let arr = [1, 2, 3, 4, 5];
    /// // checked at compile time!
    /// assert_eq!(arr.array_slice::<0, 2>(), [1, 2]);
    /// ```
    fn array_slice<const FROM: usize, const LEN: usize>(&self) -> [u8; LEN];
}

impl<const N: usize> ByteArrayExt<N> for [u8; N] {
    fn array_slice<const FROM: usize, const LEN: usize>(&self) -> [u8; LEN] {
        const_assert!(FROM: usize, LEN: usize, N: usize => FROM + LEN <= N);

        unsafe { *addr_of!(self[FROM..(FROM + LEN)]).cast::<[u8; LEN]>() }
    }
}

#[test]
fn array_slice() {
    let arr = [1, 2, 3, 4, 5];

    assert_eq!(arr.array_slice::<0, 2>(), [1, 2]);
    assert_eq!(arr.array_slice::<1, 1>(), [2]);
    assert_eq!(arr.array_slice::<4, 1>(), [5]);
    assert_eq!(arr.array_slice::<0, 0>(), [0; 0]);
    assert_eq!(arr.array_slice::<5, 0>(), [0; 0]);
}

#[cfg(test)]
#[allow(dead_code)]
mod debug {
    fn debug<T: core::fmt::Debug>(t: T) -> String {
        format!("{t:?}")
    }

    mod structs {
        use core::marker::PhantomData;

        use super::debug;

        #[test]
        fn empty() {
            #[derive(::macros::Debug)]
            struct T;

            assert_eq!(debug(T), "T");
        }

        #[test]
        fn field_named_nested() {
            #[derive(::macros::Debug)]
            struct T {
                #[debug("{}", serde_utils::to_hex(&field.field))]
                field: Inner,
            }

            struct Inner {
                field: Vec<u8>,
            }

            assert_eq!(
                debug(T {
                    field: Inner {
                        field: vec![1, 2, 3]
                    }
                }),
                "T { field: 0x010203 }"
            );
        }

        #[test]
        fn field_named() {
            #[derive(::macros::Debug)]
            struct T {
                #[debug("{}", serde_utils::to_hex(&field))]
                field: Vec<u8>,
            }

            assert_eq!(
                debug(T {
                    field: vec![1, 2, 3]
                }),
                "T { field: 0x010203 }"
            );
        }

        #[test]
        fn field_named_skip() {
            #[derive(::macros::Debug)]
            struct T {
                #[debug("{}", serde_utils::to_hex(&field))]
                field: Vec<u8>,
                #[debug(skip)]
                skip: PhantomData<()>,
            }

            assert_eq!(
                debug(T {
                    field: vec![1, 2, 3],
                    skip: PhantomData,
                }),
                "T { field: 0x010203 }"
            );
        }

        #[test]
        fn field_named_skip_generics() {
            #[derive(::macros::Debug)]
            struct T<U> {
                #[debug("{}", serde_utils::to_hex(&field))]
                field: Vec<u8>,
                #[debug(skip)]
                skip: PhantomData<U>,
            }

            assert_eq!(
                debug(T::<u8> {
                    field: vec![1, 2, 3],
                    skip: PhantomData,
                }),
                "T { field: 0x010203 }"
            );
        }

        #[test]
        fn field_named_generics() {
            #[derive(::macros::Debug)]
            struct T<U> {
                #[debug("{}", serde_utils::to_hex(&field))]
                field: Vec<u8>,
                skip: Vec<U>,
            }

            assert_eq!(
                debug(T::<u8> {
                    field: vec![1, 2, 3],
                    skip: vec![1],
                }),
                "T { field: 0x010203, skip: [1] }"
            );
        }

        #[test]
        fn field_named_container() {
            #[derive(::macros::Debug)]
            #[debug("{}", serde_utils::to_hex(&self.field))]
            struct T {
                field: Vec<u8>,
            }

            assert_eq!(
                debug(T {
                    field: vec![1, 2, 3]
                }),
                "0x010203"
            );
        }

        #[test]
        fn field_unnamed() {
            #[derive(::macros::Debug)]
            struct T(#[debug("{}", serde_utils::to_hex(&_0))] Vec<u8>);

            assert_eq!(debug(T(vec![1, 2, 3])), "T(0x010203)");
        }

        #[test]
        fn field_unnamed_container() {
            #[derive(::macros::Debug)]
            #[debug("{}", serde_utils::to_hex(&self.0))]
            struct T(Vec<u8>);

            assert_eq!(debug(T(vec![1, 2, 3])), "0x010203");
        }

        #[test]
        fn tuple_empty_container() {
            #[derive(::macros::Debug)]
            #[debug("container")]
            struct T();

            assert_eq!(debug(T()), "container");
        }

        #[test]
        fn tuple_empty() {
            #[derive(::macros::Debug)]
            struct T();

            assert_eq!(debug(T()), "T");
        }

        #[test]
        fn named_empty_container() {
            #[derive(::macros::Debug)]
            #[debug("container")]
            struct T {}

            assert_eq!(debug(T {}), "container");
        }

        #[test]
        fn empty_container() {
            #[derive(::macros::Debug)]
            #[debug("container")]
            struct T;

            assert_eq!(debug(T), "container");
        }
    }

    mod enums {
        use super::debug;

        #[test]
        fn empty() {
            #[derive(::macros::Debug)]
            enum E {
                T,
            }

            assert_eq!(debug(E::T), "T");
        }

        #[test]
        fn field_named() {
            #[derive(::macros::Debug)]
            enum E {
                T {
                    #[debug("{}", serde_utils::to_hex(&field))]
                    field: Vec<u8>,
                },
            }

            assert_eq!(
                debug(E::T {
                    field: vec![1, 2, 3]
                }),
                "T { field: 0x010203 }"
            );
        }

        #[test]
        fn field_unnamed() {
            #[derive(::macros::Debug)]
            enum E {
                T(#[debug("{}", serde_utils::to_hex(&_0))] Vec<u8>),
            }

            assert_eq!(debug(E::T(vec![1, 2, 3])), "T(0x010203)");
        }

        #[test]
        fn tuple_empty_container() {
            #[derive(::macros::Debug)]
            #[debug("container")]
            enum E {
                T(),
            }

            assert_eq!(debug(E::T()), "container");
        }

        #[test]
        fn tuple_empty() {
            #[derive(::macros::Debug)]
            enum E {
                T(),
            }

            assert_eq!(debug(E::T()), "T");
        }

        #[test]
        fn named_empty_container() {
            #[derive(::macros::Debug)]
            #[debug("container")]
            enum E {
                T {},
            }

            assert_eq!(debug(E::T {}), "container");
        }

        #[test]
        fn empty_container() {
            #[derive(::macros::Debug)]
            #[debug("container")]
            enum E {
                T,
            }

            assert_eq!(debug(E::T), "container");
        }
    }
}
