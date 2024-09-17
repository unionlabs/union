#![recursion_limit = "256"]
//! Derive macro for the `ssz::Ssz`.
//!
//! ## Attributes
//!
//! The following struct/enum attributes are available:
//!
//! - `#[ssz(union)]`: encodes and decodes an `enum` with a one-byte variant selector.
//! - `#[ssz(transparent)]`: encodes and decodes a `struct` with exactly one field as if the outermost `struct` does not exist.
//!
//! The following field attributes are available:
//!
//! ## Examples
//!
//! ### Structs
//!
//! ```rust
//! use ssz::{Ssz};
//! use ssz::types::{typenum::U4, List};
//!
//! /// Represented as an SSZ "list" wrapped in an SSZ "container".
//! #[derive(Debug, PartialEq, Ssz)]
//! struct TypicalStruct {
//!     foo: List<u8, U4>
//! }
//!
//! assert_eq!(
//!     TypicalStruct { foo: vec![42].try_into().unwrap() }.as_ssz_bytes(),
//!     vec![4, 0, 0, 0, 42]
//! );
//!
//! assert_eq!(
//!     TypicalStruct::from_ssz_bytes(&[4, 0, 0, 0, 42]).unwrap(),
//!     TypicalStruct { foo: vec![42].try_into().unwrap() },
//! );
//!
//! /// Represented as an SSZ "list" *without* an SSZ "container".
//! #[derive(Ssz)]
//! #[ssz(transparent)]
//! struct WrapperStruct {
//!     foo: List<u8, U4>
//! }
//!
//! assert_eq!(
//!     WrapperStruct { foo: vec![42].try_into().unwrap() }.as_ssz_bytes(),
//!     vec![42]
//! );
//!
//! /// Represented as an SSZ "list" *without* an SSZ "container". The `bar` byte is ignored.
//! #[derive(Debug, PartialEq, Ssz)]
//! #[ssz(transparent)]
//! struct WrapperStructSkippedField {
//!     foo: List<u8, U4>,
//!     #[ssz(skip_serializing, skip_deserializing)]
//!     bar: u8,
//! }
//!
//! assert_eq!(
//!     WrapperStructSkippedField { foo: vec![42].try_into().unwrap(), bar: 99 }.as_ssz_bytes(),
//!     vec![42]
//! );
//! assert_eq!(
//!     WrapperStructSkippedField::from_ssz_bytes(&[42]).unwrap(),
//!     WrapperStructSkippedField { foo: vec![42].try_into().unwrap(), bar: 0 }
//! );
//!
//! /// Represented as an SSZ "list" *without* an SSZ "container".
//! #[derive(Ssz)]
//! #[ssz(transparent)]
//! struct NewType(List<u8, U4>);
//!
//! assert_eq!(
//!     NewType(vec![42].try_into().unwrap()).as_ssz_bytes(),
//!     vec![42]
//! );
//!
//! /// Represented as an SSZ "list" *without* an SSZ "container". The `bar` byte is ignored.
//! #[derive(Debug, PartialEq, Ssz)]
//! #[ssz(transparent)]
//! struct NewTypeSkippedField(List<u8, U4>, #[ssz(skip_serializing, skip_deserializing)] u8);
//!
//! assert_eq!(
//!     NewTypeSkippedField(vec![42].try_into().unwrap(), 99).as_ssz_bytes(),
//!     vec![42]
//! );
//! assert_eq!(
//!     NewTypeSkippedField::from_ssz_bytes(&[42]).unwrap(),
//!     NewTypeSkippedField(vec![42].try_into().unwrap(), 0)
//! );
//! ```
//!
//! ### Enums
//!
//! ```rust
//! use ssz::{Ssz};
//! use ssz::types::{typenum::U4, List};
//!
//! /// Represented as an SSZ "union".
//! #[derive(Debug, PartialEq, Ssz)]
//! #[ssz(union)]
//! enum UnionEnum {
//!     Foo(u8),
//!     Bar(List<u8, U4>),
//! }
//!
//! assert_eq!(
//!     UnionEnum::Foo(42).as_ssz_bytes(),
//!     vec![0, 42]
//! );
//! assert_eq!(
//!     UnionEnum::from_ssz_bytes(&[1, 42, 42]).unwrap(),
//!     UnionEnum::Bar(vec![42, 42].try_into().unwrap()),
//! );
//!
//! /// Represented as only the value in the enum variant.
//! #[derive(Debug, PartialEq, Encode)]
//! #[ssz(transparent)]
//! enum TransparentEnum {
//!     Foo(u8),
//!     Bar(List<u8, U4>),
//! }
//!
//! assert_eq!(
//!     TransparentEnum::Foo(42).as_ssz_bytes(),
//!     vec![42]
//! );
//! assert_eq!(
//!     TransparentEnum::Bar(vec![42, 42].try_into().unwrap()).as_ssz_bytes(),
//!     vec![42, 42]
//! );

use syn::{parse_macro_input, DeriveInput};

use crate::ssz_trait::do_ssz;

mod ssz_trait;

/// The highest possible union selector value (higher values are reserved for backwards compatible
/// extensions).
const MAX_UNION_SELECTOR: u8 = 127;

#[proc_macro_derive(Ssz, attributes(ssz))]
pub fn ssz(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_ssz(parse_macro_input!(ts as DeriveInput))
        // .inspect(|ts| {
        //     dbg!(ts.to_string());
        // })
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
