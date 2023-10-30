//! A WebAssembly `Compiler` implementation using Cranelift.
//!
//! Cranelift is a fast IR generator created by Mozilla for usage in
//! Firefox as a next JS compiler generator.
//!
//! Compared to LLVM, Cranelift is a bit faster and made entirely in Rust.
#![deny(missing_docs, trivial_numeric_casts, unused_extern_crates)]
#![warn(unused_import_braces)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::new_without_default, clippy::new_without_default)
)]
#![cfg_attr(
    feature = "cargo-clippy",
    warn(
        clippy::float_arithmetic,
        clippy::mut_mut,
        clippy::nonminimal_bool,
        clippy::map_unwrap_or,
        clippy::print_stdout,
        clippy::unicode_not_nfc,
        clippy::use_self
    )
)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc as std;
#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
use hashbrown::{
    hash_map,
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};
#[cfg(feature = "std")]
use std::collections::{
    hash_map,
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};

mod address_map;
mod compiler;
mod config;
mod debug;
#[cfg(feature = "unwind")]
mod dwarf;
mod func_environ;
mod trampoline;
mod translator;

pub use crate::compiler::CraneliftCompiler;
pub use crate::config::{Cranelift, CraneliftOptLevel};
pub use crate::debug::{ModuleInfoMemoryOffset, ModuleInfoVmctxInfo, ValueLabelsRanges};
pub use crate::trampoline::make_trampoline_function_call;

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
