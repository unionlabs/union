//! Provides encoding (serialization) and decoding (deserialization) in the SimpleSerialize (SSZ)
//! format designed for use in Ethereum 2.0.
//!
//! Adheres to the Ethereum 2.0 [SSZ
//! specification](https://github.com/ethereum/eth2.0-specs/blob/v0.12.1/ssz/simple-serialize.md)
//! at v0.12.1.
//!
//! ## Example
//!
//! ```rust
//! use ssz::{Decode, Encode};
//! use ssz::types::{typenum::U8, VariableList};
//!
//! #[derive(PartialEq, Debug, Encode, Decode)]
//! struct Foo {
//!     a: u64,
//!     b: VariableList<u16, U8>,
//! }
//!
//! fn ssz_encode_decode_example() {
//!     let foo = Foo {
//!         a: 42,
//!         b: vec![1, 3, 3, 7].try_into().unwrap()
//!     };
//!
//!     let ssz_bytes: Vec<u8> = foo.as_ssz_bytes();
//!
//!     let decoded_foo = Foo::from_ssz_bytes(&ssz_bytes).unwrap();
//!
//!     assert_eq!(foo, decoded_foo);
//! }
//!
//! ```
//!
//! See `examples/` for manual implementations of the `Encode` and `Decode` traits.

mod decode;
mod encode;
pub mod legacy;
mod union_selector;

pub use decode::{
    impls::decode_list_of_variable_length_items, read_offset, split_union_bytes,
    try_from_iter::TryFromIter, Decode, DecodeError, SszDecoder, SszDecoderBuilder,
};
pub use encode::{
    encode_length,
    impls::{sequence_ssz_append, sequence_ssz_bytes_len},
    Encode, SszEncoder,
};
pub use union_selector::UnionSelector;

extern crate ssz_derive;
pub use ssz_derive::*;

pub mod types;

/// The number of bytes used to represent an offset.
pub const BYTES_PER_LENGTH_OFFSET: usize = 4;
// REVIEW: Why not just use usize?
/// The maximum value that can be represented using `BYTES_PER_LENGTH_OFFSET`.
#[cfg(target_pointer_width = "32")]
pub const MAX_LENGTH_VALUE: usize = (std::u32::MAX >> (8 * (4 - BYTES_PER_LENGTH_OFFSET))) as usize;
#[cfg(target_pointer_width = "64")]
pub const MAX_LENGTH_VALUE: usize = (std::u64::MAX >> (8 * (8 - BYTES_PER_LENGTH_OFFSET))) as usize;

/// The number of bytes used to indicate the variant of a union.
pub const BYTES_PER_UNION_SELECTOR: usize = 1;
/// The highest possible union selector value (higher values are reserved for backwards compatible
/// extensions).
pub const MAX_UNION_SELECTOR: u8 = 127;

/// Convenience function to SSZ encode an object supporting ssz::Encode.
///
/// Equivalent to `val.as_ssz_bytes()`.
pub fn ssz_encode<T>(val: &T) -> Vec<u8>
where
    T: Encode,
{
    val.as_ssz_bytes()
}
