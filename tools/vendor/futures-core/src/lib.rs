//! Core traits and types for asynchronous operations in Rust.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
// It cannot be included in the published code because this lints have false positives in the minimum required version.
#![cfg_attr(test, warn(single_use_lifetimes))]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod future;
#[doc(no_inline)]
pub use self::future::{FusedFuture, Future, TryFuture};

pub mod stream;
#[doc(no_inline)]
pub use self::stream::{FusedStream, Stream, TryStream};

#[macro_use]
pub mod task;
