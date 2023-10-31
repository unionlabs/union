// Copyright (C) 2018-2021 Stephane Raux. Distributed under the 0BSD license.

//! # Overview
//! - [📦 crates.io](https://crates.io/crates/enum-iterator)
//! - [📖 Documentation](https://docs.rs/enum-iterator)
//! - [⚖ 0BSD license](https://spdx.org/licenses/0BSD.html)
//!
//! Tools to iterate over the variants of a field-less enum.
//!
//! See the [`IntoEnumIterator`] trait.
//!
//! # Example
//! ```
//! use enum_iterator::IntoEnumIterator;
//!
//! #[derive(Debug, IntoEnumIterator, PartialEq)]
//! enum Day { Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday }
//!
//! fn main() {
//!     assert_eq!(Day::into_enum_iter().next(), Some(Day::Monday));
//!     assert_eq!(Day::into_enum_iter().last(), Some(Day::Sunday));
//! }
//! ```
//!
//! # Contribute
//! All contributions shall be licensed under the [0BSD license](https://spdx.org/licenses/0BSD.html).

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

pub use enum_iterator_derive::IntoEnumIterator;

use core::iter;

/// Trait to iterate over the variants of a field-less enum.
///
/// Field-less (a.k.a. C-like) enums are enums whose variants don't have additional data.
///
/// This trait is meant to be derived.
///
/// # Example
///
/// ```
/// use enum_iterator::IntoEnumIterator;
///
/// #[derive(Clone, IntoEnumIterator, PartialEq)]
/// enum Direction { North, South, West, East }
///
/// fn main() {
///     assert_eq!(Direction::VARIANT_COUNT, 4);
///     assert!(Direction::into_enum_iter().eq([Direction::North,
///         Direction::South, Direction::West, Direction::East].iter()
///         .cloned()));
/// }
/// ```
pub trait IntoEnumIterator: Sized {
    /// Type of the iterator over the variants.
    type Iterator: Iterator<Item = Self> + iter::ExactSizeIterator + iter::FusedIterator + Copy;

    /// Number of variants.
    const VARIANT_COUNT: usize;

    /// Returns an iterator over the variants.
    ///
    /// Variants are yielded in the order they are defined in the enum.
    fn into_enum_iter() -> Self::Iterator;
}
