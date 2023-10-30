#![no_std]
#![deny(missing_docs)]
#![allow(clippy::missing_safety_doc)] // The safety requirement is "use the procedural derive".
#![allow(clippy::needless_range_loop)] // range loop style is clearer in most places in enumset
#![cfg_attr(docsrs, feature(doc_cfg))]

//! A library for defining enums that can be used in compact bit sets. It supports arbitrarily
//! large enums, and has very basic support for using them in constants.
//!
//! The following feature flags may be used for this crate:
//!
//! * `serde` enables serialization support for [`EnumSet`].
//! * `alloc` enables functions that require allocation.
//!
//! # Defining enums for use with EnumSet
//!
//! Enums to be used with [`EnumSet`] should be defined using `#[derive(EnumSetType)]`:
//!
//! ```rust
//! # use enumset::*;
//! #[derive(EnumSetType, Debug)]
//! pub enum Enum {
//!    A, B, C, D, E, F, G,
//! }
//! ```
//!
//! For more information on more advanced use cases, see the documentation for
//! [`#[derive(EnumSetType)]`](./derive.EnumSetType.html).
//!
//! # Working with EnumSets
//!
//! EnumSets can be constructed via [`EnumSet::new()`] like a normal set. In addition,
//! `#[derive(EnumSetType)]` creates operator overloads that allow you to create EnumSets like so:
//!
//! ```rust
//! # use enumset::*;
//! # #[derive(EnumSetType, Debug)] pub enum Enum { A, B, C, D, E, F, G }
//! let new_set = Enum::A | Enum::C | Enum::G;
//! assert_eq!(new_set.len(), 3);
//! ```
//!
//! All bitwise operations you would expect to work on bitsets also work on both EnumSets and
//! enums with `#[derive(EnumSetType)]`:
//! ```rust
//! # use enumset::*;
//! # #[derive(EnumSetType, Debug)] pub enum Enum { A, B, C, D, E, F, G }
//! // Intersection of sets
//! assert_eq!((Enum::A | Enum::B) & Enum::C, EnumSet::empty());
//! assert_eq!((Enum::A | Enum::B) & Enum::A, Enum::A);
//! assert_eq!(Enum::A & Enum::B, EnumSet::empty());
//!
//! // Symmetric difference of sets
//! assert_eq!((Enum::A | Enum::B) ^ (Enum::B | Enum::C), Enum::A | Enum::C);
//! assert_eq!(Enum::A ^ Enum::C, Enum::A | Enum::C);
//!
//! // Difference of sets
//! assert_eq!((Enum::A | Enum::B | Enum::C) - Enum::B, Enum::A | Enum::C);
//!
//! // Complement of sets
//! assert_eq!(!(Enum::E | Enum::G), Enum::A | Enum::B | Enum::C | Enum::D | Enum::F);
//! ```
//!
//! The [`enum_set!`] macro allows you to create EnumSets in constant contexts:
//!
//! ```rust
//! # use enumset::*;
//! # #[derive(EnumSetType, Debug)] pub enum Enum { A, B, C, D, E, F, G }
//! const CONST_SET: EnumSet<Enum> = enum_set!(Enum::A | Enum::B);
//! assert_eq!(CONST_SET, Enum::A | Enum::B);
//! ```
//!
//! Mutable operations on the [`EnumSet`] otherwise similarly to Rust's builtin sets:
//!
//! ```rust
//! # use enumset::*;
//! # #[derive(EnumSetType, Debug)] pub enum Enum { A, B, C, D, E, F, G }
//! let mut set = EnumSet::new();
//! set.insert(Enum::A);
//! set.insert_all(Enum::E | Enum::G);
//! assert!(set.contains(Enum::A));
//! assert!(!set.contains(Enum::B));
//! assert_eq!(set, Enum::A | Enum::E | Enum::G);
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

mod macros;

mod repr;
mod set;
mod traits;

pub use crate::macros::__internal;
pub use crate::set::{EnumSet, EnumSetIter};
pub use crate::traits::{EnumSetType, EnumSetTypeWithRepr};

/// The procedural macro used to derive [`EnumSetType`], and allow enums to be used with
/// [`EnumSet`].
///
/// # Limitations
///
/// Currently, the following limitations apply to what kinds of enums this macro may be used with:
///
/// * The enum must have no data fields in any variant.
/// * Variant discriminators must be zero or positive.
/// * No variant discriminator may be larger than `0xFFFFFFBF`. This is chosen to limit problems
///   involving overflow and similar edge cases.
/// * Variant discriminators must be defined with integer literals. Expressions like `V = 1 + 1`
///   are not currently supported.
///
/// # Additional Impls
///
/// In addition to the implementation of `EnumSetType`, this procedural macro creates multiple
/// other impls that are either required for the macro to work, or make the procedural macro more
/// ergonomic to use.
///
/// A full list of traits implemented as is follows:
///
/// * [`Copy`], [`Clone`], [`Eq`], [`PartialEq`] implementations are created to allow `EnumSet`
///   to function properly. These automatic implementations may be suppressed using
///   `#[enumset(no_super_impls)]`, but these traits must still be implemented in another way.
/// * [`PartialEq`], [`Sub`], [`BitAnd`], [`BitOr`], [`BitXor`], and [`Not`] implementations are
///   created to allow the crate to be used more ergonomically in expressions. These automatic
///   implementations may be suppressed using `#[enumset(no_ops)]`.
///
/// # Options
///
/// Options are given with `#[enumset(foo)]` annotations attached to the same enum as the derive.
/// Multiple options may be given in the same annotation using the `#[enumset(foo, bar)]` syntax.
///
/// A full list of options is as follows:
///
/// * `#[enumset(no_super_impls)]` prevents the derive from creating implementations required for
///   [`EnumSet`] to function. When this attribute is specified, implementations of [`Copy`],
///   [`Clone`], [`Eq`], and [`PartialEq`]. This can be useful if you are using a code generator
///   that already derives these traits. These impls should function identically to the
///   automatically derived versions, or unintentional behavior may be a result.
/// * `#[enumset(no_ops)` prevents the derive from implementing any operator traits.
/// * `#[enumset(crate_name = "enumset2")]` may be used to change the name of the `enumset` crate
///   used in the generated code. When the `std` feature is enabled, enumset parses `Cargo.toml`
///   to determine the name of the crate, and this flag is unnecessary.
/// * `#[enumset(repr = "u8")]` may be used to specify the in-memory representation of `EnumSet`s
///   of this enum type. The effects of this are described in [the `EnumSet` documentation under
///   “FFI, Safety and `repr`”][EnumSet#ffi-safety-and-repr]. Allowed types are `u8`, `u16`, `u32`,
///   `u64` and `u128`. If this is not used, then the derive macro will choose a type to best fit
///   the enum, but there are no guarantees about which type will be chosen.
/// * `#[enumset(repr = "array")]` forces the `EnumSet` of this type to be backed with an array,
///   even if all the variants could fit into a primitive numeric type.
///
/// When the `serde` feature is used, the following features may also be specified. These options
/// may be used (with no effect) when building without the feature enabled:
///
/// * `#[enumset(serialize_repr = "…")]` may be used to override the way the `EnumSet` is
///   serialized. Valid options are `u8`, `u16`, `u32`, `u64`, `list`, `map` and `array`. For more
///   information, see the ["Serialization" section of the `EnumSet` documentation]
///   (EnumSet#serialization).
/// * `#[enumset(serialize_deny_unknown)]` causes the generated deserializer to return an error
///   for unknown bits instead of silently ignoring them.
///
/// # Examples
///
/// Deriving a plain EnumSetType:
///
/// ```rust
/// # use enumset::*;
/// #[derive(EnumSetType)]
/// pub enum Enum {
///    A, B, C, D, E, F, G,
/// }
/// ```
///
/// Deriving a sparse EnumSetType:
///
/// ```rust
/// # use enumset::*;
/// #[derive(EnumSetType)]
/// pub enum SparseEnum {
///    A = 10, B = 20, C = 30, D = 127,
/// }
/// ```
///
/// Deriving an EnumSetType without adding ops:
///
/// ```rust
/// # use enumset::*;
/// #[derive(EnumSetType)]
/// #[enumset(no_ops)]
/// pub enum NoOpsEnum {
///    A, B, C, D, E, F, G,
/// }
/// ```
///
/// [`Sub`]: core::ops::Sub
/// [`BitAnd`]: core::ops::BitAnd
/// [`BitOr`]: core::ops::BitOr
/// [`BitXor`]: core::ops::BitXor
/// [`Not`]: core::ops::Not
pub use enumset_derive::EnumSetType;
