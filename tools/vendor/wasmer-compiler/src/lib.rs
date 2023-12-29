//! The `wasmer-compiler` crate provides the necessary abstractions
//! to create a compiler.
//!
//! It provides an universal way of parsing a module via `wasmparser`,
//! while giving the responsibility of compiling specific function
//! WebAssembly bodies to the `Compiler` implementation.

#![deny(missing_docs, trivial_numeric_casts, unused_extern_crates)]
#![warn(unused_import_braces)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::new_without_default, clippy::upper_case_acronyms)
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
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(all(feature = "std", feature = "core"))]
compile_error!(
    "The `std` and `core` features are both enabled, which is an error. Please enable only once."
);

#[cfg(all(not(feature = "std"), not(feature = "core")))]
compile_error!("Both the `std` and `core` features are disabled. Please enable one of them.");

#[cfg(feature = "core")]
extern crate alloc;

mod lib {
    #[cfg(feature = "core")]
    pub mod std {
        pub use alloc::{borrow, boxed, str, string, sync, vec};
        pub use core::fmt;
        pub use hashbrown as collections;
    }

    #[cfg(feature = "std")]
    pub mod std {
        pub use std::{borrow, boxed, collections, fmt, str, string, sync, vec};
    }
}

mod engine;
mod traits;

pub use crate::engine::*;
pub use crate::traits::*;

#[cfg(feature = "translator")]
mod artifact_builders;

#[cfg(feature = "translator")]
pub use self::artifact_builders::*;

#[cfg(feature = "translator")]
mod compiler;

#[cfg(feature = "translator")]
#[macro_use]
mod translator;
#[cfg(feature = "translator")]
pub use crate::compiler::{Compiler, CompilerConfig};
#[cfg(feature = "translator")]
pub use crate::translator::{
    from_binaryreadererror_wasmerror, translate_module, wptype_to_type, FunctionBinaryReader,
    FunctionBodyData, FunctionMiddleware, MiddlewareBinaryReader, MiddlewareReaderState,
    ModuleEnvironment, ModuleMiddleware, ModuleMiddlewareChain, ModuleTranslationState,
};

pub use wasmer_types::{Addend, CodeOffset, Features};

#[cfg(feature = "translator")]
/// wasmparser is exported as a module to slim compiler dependencies
pub use wasmparser;

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
