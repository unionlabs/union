// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A Big integer (signed version: `BigInt`, unsigned version: `BigUint`).
//!
//! A `BigUint` is represented as a vector of `BigDigit`s.
//! A `BigInt` is a combination of `BigUint` and `Sign`.
//!
//! Common numerical operations are overloaded, so we can treat them
//! the same way we treat other numbers.
//!
//! ## Example
//!
//! ```rust
//! # fn main() {
//! use num_bigint::BigUint;
//! use num_traits::{Zero, One};
//! use std::mem::replace;
//!
//! // Calculate large fibonacci numbers.
//! fn fib(n: usize) -> BigUint {
//!     let mut f0: BigUint = Zero::zero();
//!     let mut f1: BigUint = One::one();
//!     for _ in 0..n {
//!         let f2 = f0 + &f1;
//!         // This is a low cost way of swapping f0 with f1 and f1 with f2.
//!         f0 = replace(&mut f1, f2);
//!     }
//!     f0
//! }
//!
//! // This is a very large number.
//! println!("fib(1000) = {}", fib(1000));
//! # }
//! ```
//!
//! It's easy to generate large random numbers:
//!
//! ```rust,ignore
//! use num_bigint::{ToBigInt, RandBigInt};
//!
//! let mut rng = rand::thread_rng();
//! let a = rng.gen_bigint(1000);
//!
//! let low = -10000.to_bigint().unwrap();
//! let high = 10000.to_bigint().unwrap();
//! let b = rng.gen_bigint_range(&low, &high);
//!
//! // Probably an even larger number.
//! println!("{}", a * b);
//! ```
//!
//! See the "Features" section for instructions for enabling random number generation.
//!
//! ## Features
//!
//! The `std` crate feature is enabled by default, and is mandatory before Rust
//! 1.36 and the stabilized `alloc` crate.  If you depend on `num-bigint` with
//! `default-features = false`, you must manually enable the `std` feature yourself
//! if your compiler is not new enough.
//!
//! ### Random Generation
//!
//! `num-bigint` supports the generation of random big integers when the `rand`
//! feature is enabled. To enable it include rand as
//!
//! ```toml
//! rand = "0.8"
//! num-bigint = { version = "0.4", features = ["rand"] }
//! ```
//!
//! Note that you must use the version of `rand` that `num-bigint` is compatible
//! with: `0.8`.
//!
//!
//! ## Compatibility
//!
//! The `num-bigint` crate is tested for rustc 1.31 and greater.

#![doc(html_root_url = "https://docs.rs/num-bigint/0.4")]
#![warn(rust_2018_idioms)]
#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(feature = "std")]
mod std_alloc {
    pub(crate) use std::borrow::Cow;
    #[cfg(any(feature = "quickcheck"))]
    pub(crate) use std::boxed::Box;
    pub(crate) use std::string::String;
    pub(crate) use std::vec::Vec;
}

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod std_alloc {
    pub(crate) use alloc::borrow::Cow;
    #[cfg(any(feature = "quickcheck"))]
    pub(crate) use alloc::boxed::Box;
    pub(crate) use alloc::string::String;
    pub(crate) use alloc::vec::Vec;
}

use core::fmt;
#[cfg(feature = "std")]
use std::error::Error;

#[macro_use]
mod macros;

mod bigint;
mod biguint;

#[cfg(feature = "rand")]
mod bigrand;

#[cfg(target_pointer_width = "32")]
type UsizePromotion = u32;
#[cfg(target_pointer_width = "64")]
type UsizePromotion = u64;

#[cfg(target_pointer_width = "32")]
type IsizePromotion = i32;
#[cfg(target_pointer_width = "64")]
type IsizePromotion = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseBigIntError {
    kind: BigIntErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BigIntErrorKind {
    Empty,
    InvalidDigit,
}

impl ParseBigIntError {
    fn __description(&self) -> &str {
        use crate::BigIntErrorKind::*;
        match self.kind {
            Empty => "cannot parse integer from empty string",
            InvalidDigit => "invalid digit found in string",
        }
    }

    fn empty() -> Self {
        ParseBigIntError {
            kind: BigIntErrorKind::Empty,
        }
    }

    fn invalid() -> Self {
        ParseBigIntError {
            kind: BigIntErrorKind::InvalidDigit,
        }
    }
}

impl fmt::Display for ParseBigIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.__description().fmt(f)
    }
}

#[cfg(feature = "std")]
impl Error for ParseBigIntError {
    fn description(&self) -> &str {
        self.__description()
    }
}

/// The error type returned when a checked conversion regarding big integer fails.
#[cfg(has_try_from)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TryFromBigIntError<T> {
    original: T,
}

#[cfg(has_try_from)]
impl<T> TryFromBigIntError<T> {
    fn new(original: T) -> Self {
        TryFromBigIntError { original }
    }

    fn __description(&self) -> &str {
        "out of range conversion regarding big integer attempted"
    }

    /// Extract the original value, if available. The value will be available
    /// if the type before conversion was either [`BigInt`] or [`BigUint`].
    ///
    /// [`BigInt`]: struct.BigInt.html
    /// [`BigUint`]: struct.BigUint.html
    pub fn into_original(self) -> T {
        self.original
    }
}

#[cfg(all(feature = "std", has_try_from))]
impl<T> std::error::Error for TryFromBigIntError<T>
where
    T: fmt::Debug,
{
    fn description(&self) -> &str {
        self.__description()
    }
}

#[cfg(has_try_from)]
impl<T> fmt::Display for TryFromBigIntError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.__description().fmt(f)
    }
}

pub use crate::biguint::BigUint;
pub use crate::biguint::ToBigUint;
pub use crate::biguint::U32Digits;
pub use crate::biguint::U64Digits;

pub use crate::bigint::BigInt;
pub use crate::bigint::Sign;
pub use crate::bigint::ToBigInt;

#[cfg(feature = "rand")]
pub use crate::bigrand::{RandBigInt, RandomBits, UniformBigInt, UniformBigUint};

mod big_digit {
    /// A `BigDigit` is a `BigUint`'s composing element.
    #[cfg(not(u64_digit))]
    pub(crate) type BigDigit = u32;
    #[cfg(u64_digit)]
    pub(crate) type BigDigit = u64;

    /// A `DoubleBigDigit` is the internal type used to do the computations.  Its
    /// size is the double of the size of `BigDigit`.
    #[cfg(not(u64_digit))]
    pub(crate) type DoubleBigDigit = u64;
    #[cfg(u64_digit)]
    pub(crate) type DoubleBigDigit = u128;

    /// A `SignedDoubleBigDigit` is the signed version of `DoubleBigDigit`.
    #[cfg(not(u64_digit))]
    pub(crate) type SignedDoubleBigDigit = i64;
    #[cfg(u64_digit)]
    pub(crate) type SignedDoubleBigDigit = i128;

    // `DoubleBigDigit` size dependent
    #[cfg(not(u64_digit))]
    pub(crate) const BITS: u8 = 32;
    #[cfg(u64_digit)]
    pub(crate) const BITS: u8 = 64;

    pub(crate) const HALF_BITS: u8 = BITS / 2;
    pub(crate) const HALF: BigDigit = (1 << HALF_BITS) - 1;

    const LO_MASK: DoubleBigDigit = (1 << BITS) - 1;
    pub(crate) const MAX: BigDigit = LO_MASK as BigDigit;

    #[inline]
    fn get_hi(n: DoubleBigDigit) -> BigDigit {
        (n >> BITS) as BigDigit
    }
    #[inline]
    fn get_lo(n: DoubleBigDigit) -> BigDigit {
        (n & LO_MASK) as BigDigit
    }

    /// Split one `DoubleBigDigit` into two `BigDigit`s.
    #[inline]
    pub(crate) fn from_doublebigdigit(n: DoubleBigDigit) -> (BigDigit, BigDigit) {
        (get_hi(n), get_lo(n))
    }

    /// Join two `BigDigit`s into one `DoubleBigDigit`
    #[inline]
    pub(crate) fn to_doublebigdigit(hi: BigDigit, lo: BigDigit) -> DoubleBigDigit {
        DoubleBigDigit::from(lo) | (DoubleBigDigit::from(hi) << BITS)
    }
}
