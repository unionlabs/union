#![doc = include_str!("../README.md")]
#![deny(
    clippy::pedantic,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::too_long_first_doc_paragraph
)]
#![feature(trait_alias)]

extern crate alloc;

use core::{
    fmt::{self, Debug, Display},
    iter,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
pub use typenum;

use crate::{
    encoding::{Decode, Encode},
    errors::{ExpectedLength, InvalidLength},
    validated::Validated,
};

pub const DELAY_PERIOD: u64 = 0;

pub use unionlabs_primitives as primitives;

/// Wrapper types around protos defined in <https://github.com/cosmos/gogoproto/tree/main/protobuf/google/protobuf>, matching the proto module structure.
pub mod google;

#[cfg(feature = "near")]
pub mod near;

/// Defines types that wrap the IBC specification, matching the proto module structure. This also includes `union` extensions to ibc (i.e. types defined in `union.ibc`).
pub mod ibc;

/// Defines types that wrap the cosmos specification, matching the proto module structure.
pub mod cosmos;

/// Defines types that wrap the cosmwasm specification, matching the proto module structure.
pub mod cosmwasm;

/// Various ethereum types. Types that have an IBC counterpart are defined in [`ibc`].
pub mod ethereum;

/// Types specific to the union protocol.
pub mod union;

/// Types specific to the berachain protocol.
pub mod berachain;

pub mod aptos;

pub mod bounded;

pub mod constants;

// TODO: Remove (only used in ucs01-relay-api currently)
pub mod validated;

pub mod encoding;

/// Stable replacement for [`!`].
pub mod never;

/// Various identifier types used throughout the IBC stack.
pub mod id;

pub mod signer;

pub mod traits;

pub(crate) mod macros;

pub mod errors;

pub mod bech32;

pub mod tuple;

#[cfg(feature = "proto")]
pub use ::prost;

#[cfg(any(feature = "test-utils", test))]
#[allow(clippy::missing_panics_doc)]
pub mod test_utils;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromProtoBytesError<E> {
    #[error("unable to convert from the raw prost type")]
    TryFromProto(#[source] E),
    #[error("unable to decode from raw proto bytes")]
    Decode(#[source] prost::DecodeError),
}

pub trait TypeUrl {
    fn type_url() -> String;
}

pub trait Msg: Clone + Encode<encoding::Proto> + TypeUrl {
    type Response: Decode<encoding::Proto, Error: core::error::Error> + TypeUrl;
}

#[cfg(feature = "ethabi")]
#[derive(Debug, thiserror::Error)]
// TODO: Rename this once we fully remove ethers
pub enum TryFromEthAbiBytesErrorAlloy<E> {
    #[error(transparent)]
    Convert(E),
    #[error("unable to decode from raw ethabi bytes")]
    Decode(#[from] alloy::core::sol_types::Error),
}

/// An empty string. Will only parse/serialize to/from `""`.
pub type EmptyString<S = String> = Validated<S, EmptyStringValidator>;
pub struct EmptyStringValidator;

impl<T: AsRef<str>> validated::Validate<T> for EmptyStringValidator {
    type Error = InvalidLength;

    fn validate(s: T) -> Result<T, Self::Error> {
        if s.as_ref().is_empty() {
            Ok(s)
        } else {
            Err(InvalidLength {
                expected: ExpectedLength::Exact(0),
                found: s.as_ref().len(),
            })
        }
    }
}

#[doc(hidden)]
pub use paste::paste;

#[macro_export]
macro_rules! export_wasm_client_type {
    ($type:ident) => {
        const _: $crate::WasmClientType = $crate::WasmClientType::$type;
        $crate::paste! {
            #[no_mangle]
            #[used]
            #[allow(non_upper_case_globals)]
            static [ <WASM_CLIENT_TYPE_ $type> ]: u8 = 0;
        }
    };
}

/// This type is used to discriminate 08-wasm light clients.
///
/// We need to be able to determine the light client from the light client code itself (not instantiated yet).
/// Light clients supported by voyager must export a `#[no_mangle] static WASM_CLIENT_TYPE_<TYPE>: u8 = 0` variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WasmClientType {
    Cometbls,
    Tendermint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    Wasm(WasmClientType),
    Tendermint,
    Cometbls,
    _11Cometbls,
}

impl ClientType {
    #[must_use]
    pub const fn identifier_prefix(self) -> &'static str {
        match self {
            ClientType::Wasm(_) => "08-wasm",
            ClientType::Tendermint => "07-tendermint",
            ClientType::Cometbls => "cometbls",
            ClientType::_11Cometbls => "11-cometbls",
        }
    }
}

impl FromStr for WasmClientType {
    type Err = WasmClientTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Cometbls" => Ok(WasmClientType::Cometbls),
            "Tendermint" => Ok(WasmClientType::Tendermint),
            _ => Err(WasmClientTypeParseError::UnknownType(s.to_string())),
        }
    }
}

impl Display for WasmClientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cometbls => write!(f, "Cometbls"),
            Self::Tendermint => write!(f, "Tendermint"),
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

pub fn ensure<E>(expr: bool, err: E) -> Result<(), E> {
    expr.then_some(()).ok_or(err)
}

pub struct ErrorReporter<T: core::error::Error>(pub T);

impl<T: core::error::Error> ErrorReporter<T> {
    pub fn with_message(&self, message: &str) -> String {
        format!("{message}: {self}")
    }
}

impl<T: core::error::Error> Display for ErrorReporter<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)?;

        for e in iter::successors(self.0.source(), |e| (*e).source()) {
            write!(f, ": {e}")?;
        }

        Ok(())
    }
}

#[must_use = "constructing an iterator has no effect"]
pub struct BytesBitIterator<'a> {
    bz: &'a [u8],
    pos: core::ops::Range<usize>,
}

impl<'a> BytesBitIterator<'a> {
    pub fn new(bz: &'a impl AsRef<[u8]>) -> Self {
        BytesBitIterator {
            bz: bz.as_ref(),
            pos: (0..bz.as_ref().len() * 8),
        }
    }

    /// Returns the `index`-th bit in the bytes.
    fn get_bit(&self, index: usize) -> bool {
        // debug_assert_eq!(self.hash_bytes.len(), Hash::LENGTH); // invariant
        // debug_assert_lt!(index, Hash::LENGTH_IN_BITS); // assumed precondition
        let pos = index / 8;
        let bit = index % 8;
        (self.bz[pos] >> bit) & 1 != 0
    }
}

impl core::iter::Iterator for BytesBitIterator<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos.next().map(|x| self.get_bit(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pos.size_hint()
    }
}

impl core::iter::DoubleEndedIterator for BytesBitIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos.next_back().map(|x| self.get_bit(x))
    }
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
