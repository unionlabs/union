//! Generic Artifact abstraction for Wasmer Engines.

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
        clippy::option_map_unwrap_or,
        clippy::option_map_unwrap_or_else,
        clippy::print_stdout,
        clippy::unicode_not_nfc,
        clippy::use_self
    )
)]

mod artifact;
mod error;
mod funcbody;

pub use crate::artifact::{ArtifactCreate, MetadataHeader, Upcastable};
pub use crate::error::{DeserializeError, ImportError, PreInstantiationError, SerializeError};
pub use crate::funcbody::VMFunctionBody;
use loupe::MemoryUsage;

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A safe wrapper around `VMFunctionBody`.
#[derive(Clone, Copy, Debug, MemoryUsage)]
#[repr(transparent)]
pub struct FunctionBodyPtr(pub *const VMFunctionBody);

impl std::ops::Deref for FunctionBodyPtr {
    type Target = *const VMFunctionBody;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// # Safety
/// The VMFunctionBody that this points to is opaque, so there's no data to
/// read or write through this pointer. This is essentially a usize.
unsafe impl Send for FunctionBodyPtr {}
/// # Safety
/// The VMFunctionBody that this points to is opaque, so there's no data to
/// read or write through this pointer. This is essentially a usize.
unsafe impl Sync for FunctionBodyPtr {}
