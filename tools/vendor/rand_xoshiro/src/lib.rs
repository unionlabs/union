// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate implements the [xoshiro] family of pseudorandom number generators
//! designed by David Blackman and Sebastiano Vigna. They feature high
//! perfomance and a small state and superseed the previous xorshift-based
//! generators. However, they are no cryptographically secure and their output
//! can be predicted by observing a few samples.
//!
//! The following generators are implemented:
//!
//! # 64-bit generators
//! - [`Xoshiro256StarStar`]: Recommended for all purposes. Excellent speed and
//!   a state space (256 bits) large enough for any parallel application.
//! - [`Xoshiro256PlusPlus`]: Recommended for all purposes. Excellent speed and
//!   a state space (256 bits) large enough for any parallel application.
//! - [`Xoshiro256Plus`]: Recommended for generating 64-bit floating-point
//!   numbers. About 15% faster than `Xoshiro256StarStar`, but has a [low linear
//!   complexity] in the lowest bits (which are discarded when generating
//!   floats), making it fail linearity tests. This is unlikely to have any
//!   impact in practise.
//! - [`Xoroshiro128StarStar`]: An alternative to `Xoshiro256StarStar`, having
//!   the same speed but using half the state. Only suited for low-scale parallel
//!   applications.
//! - [`Xoroshiro128PlusPlus`]: An alternative to `Xoshiro256PlusPlus`, having
//!   the same speed but using half the state. Only suited for low-scale parallel
//!   applications.
//! - [`Xoroshiro128Plus`]: An alternative to `Xoshiro256Plus`, having the same
//!   speed but using half the state. Only suited for low-scale parallel
//!   applications. Has a [low linear complexity] in the lowest bits (which are
//!   discarded when generating floats), making it fail linearity tests. This is
//!   unlikely to have any impact in practise.
//! - [`Xoshiro512StarStar`]: An alternative to `Xoshiro256StarStar` with more
//!   state and the same speed.
//! - [`Xoshiro512PlusPlus`]: An alternative to `Xoshiro256PlusPlus` with more
//!   state and the same speed.
//! - [`Xoshiro512Plus`]: An alternative to `Xoshiro512Plus` with more
//!   state and the same speed. Has a [low linear complexity] in the lowest bits
//!   (which are discarded when generating floats), making it fail linearity
//!   tests. This is unlikely to have any impact in practise.
//! - [`SplitMix64`]: Recommended for initializing generators of the xoshiro
//!   familiy from a 64-bit seed. Used for implementing `seed_from_u64`.
//!
//! # 32-bit generators
//! - [`Xoshiro128StarStar`]: Recommended for all purposes. Excellent speed.
//! - [`Xoshiro128PlusPlus`]: Recommended for all purposes. Excellent speed.
//! - [`Xoshiro128Plus`]: Recommended for generating 32-bit floating-point
//!   numbers. Faster than `Xoshiro128StarStar`, but has a [low linear
//!   complexity] in the lowest bits (which are discarded when generating
//!   floats), making it fail linearity tests. This is unlikely to have any
//!   impact in practise.
//! - [`Xoroshiro64StarStar`]: An alternative to `Xoshiro128StarStar`, having
//!   the same speed but using half the state.
//! - [`Xoroshiro64Star`]: An alternative to `Xoshiro128Plus`, having the
//!   same speed but using half the state. Has a [low linear complexity] in the
//!   lowest bits (which are discarded when generating floats), making it fail
//!   linearity tests. This is unlikely to have any impact in practise.
//!
//! The `*PlusPlus` generators perform similarily to the `*StarStar` generators.
//! See the [xoshiro paper], where the differences are discussed in detail.
//!
//! [xoshiro]: http://xoshiro.di.unimi.it/
//! [xoshiro paper]: http://vigna.di.unimi.it/ftp/papers/ScrambledLinear.pdf
//! [low linear complexity]: http://xoshiro.di.unimi.it/lowcomp.php

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
       html_favicon_url = "https://www.rust-lang.org/favicon.ico",
       html_root_url = "https://docs.rs/rand_xoshiro/0.6.0")]

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![allow(clippy::unreadable_literal)]
#![no_std]

#[macro_use]
mod common;
mod splitmix64;
mod xoshiro128starstar;
mod xoshiro128plusplus;
mod xoshiro128plus;
mod xoshiro256starstar;
mod xoshiro256plusplus;
mod xoshiro256plus;
mod xoshiro512starstar;
mod xoshiro512plusplus;
mod xoshiro512plus;
mod xoroshiro128plus;
mod xoroshiro128plusplus;
mod xoroshiro128starstar;
mod xoroshiro64starstar;
mod xoroshiro64star;

pub use rand_core;
pub use splitmix64::SplitMix64;
pub use xoshiro128starstar::Xoshiro128StarStar;
pub use xoshiro128plusplus::Xoshiro128PlusPlus;
pub use xoshiro128plus::Xoshiro128Plus;
pub use xoshiro256starstar::Xoshiro256StarStar;
pub use xoshiro256plusplus::Xoshiro256PlusPlus;
pub use xoshiro256plus::Xoshiro256Plus;
pub use common::Seed512;
pub use xoshiro512starstar::Xoshiro512StarStar;
pub use xoshiro512plusplus::Xoshiro512PlusPlus;
pub use xoshiro512plus::Xoshiro512Plus;
pub use xoroshiro128plus::Xoroshiro128Plus;
pub use xoroshiro128starstar::Xoroshiro128StarStar;
pub use xoroshiro128plusplus::Xoroshiro128PlusPlus;
pub use xoroshiro64starstar::Xoroshiro64StarStar;
pub use xoroshiro64star::Xoroshiro64Star;
