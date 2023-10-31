//! Universal backend for Wasmer compilers.
//!
//! Given a compiler (such as `CraneliftCompiler` or `LLVMCompiler`)
//! it generates the compiled machine code, and publishes it into
//! memory so it can be used externally.

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

mod artifact;
mod builder;
mod code_memory;
mod engine;
mod link;
mod unwind;

pub use crate::artifact::UniversalArtifact;
pub use crate::builder::Universal;
pub use crate::code_memory::CodeMemory;
pub use crate::engine::UniversalEngine;
pub use crate::link::link_module;

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
