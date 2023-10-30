//! `gimli` is a library for reading and writing the
//! [DWARF debugging format](https://dwarfstd.org/).
//!
//! See the [read](./read/index.html) and [write](./write/index.html) modules
//! for examples and API documentation.
//!
//! ## Cargo Features
//!
//! Cargo features that can be enabled with `gimli`:
//!
//! * `std`: Enabled by default. Use the `std` library. Disabling this feature
//! allows using `gimli` in embedded environments that do not have access to
//! `std`. Note that even when `std` is disabled, `gimli` still requires an
//! implementation of the `alloc` crate.
//!
//! * `read`: Enabled by default. Enables the `read` module. Use of `std` is
//! optional.
//!
//! * `write`: Enabled by default. Enables the `write` module. Always uses
//! the `std` library.
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
// Selectively enable rust 2018 warnings
#![warn(bare_trait_objects)]
#![warn(unused_extern_crates)]
#![warn(ellipsis_inclusive_range_patterns)]
//#![warn(elided_lifetimes_in_paths)]
#![warn(explicit_outlives_requirements)]
// Allow clippy warnings when we aren't building with clippy.
#![allow(unknown_lints)]
// False positives with `fallible_iterator`.
#![allow(clippy::should_implement_trait)]
// Many false positives involving `continue`.
#![allow(clippy::never_loop)]
// False positives when block expressions are used inside an assertion.
#![allow(clippy::panic_params)]
#![no_std]

#[allow(unused_imports)]
#[cfg(any(feature = "read", feature = "write"))]
#[macro_use]
extern crate alloc;

#[cfg(any(feature = "std", feature = "write"))]
#[macro_use]
extern crate std;

#[cfg(feature = "stable_deref_trait")]
pub use stable_deref_trait::{CloneStableDeref, StableDeref};

mod common;
pub use crate::common::*;

mod arch;
pub use crate::arch::*;

pub mod constants;
// For backwards compat.
pub use crate::constants::*;

mod endianity;
pub use crate::endianity::{BigEndian, Endianity, LittleEndian, NativeEndian, RunTimeEndian};

pub mod leb128;

#[cfg(feature = "read-core")]
pub mod read;
// For backwards compat.
#[cfg(feature = "read-core")]
pub use crate::read::*;

#[cfg(feature = "write")]
pub mod write;

#[cfg(test)]
mod test_util;
