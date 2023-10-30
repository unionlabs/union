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
mod engine;
mod serialize;
mod trampoline;

pub use crate::artifact::UniversalArtifactBuild;
pub use crate::engine::UniversalEngineBuilder;
pub use crate::serialize::SerializableModule;
pub use crate::trampoline::*;
pub use wasmer_artifact::{ArtifactCreate, MetadataHeader, Upcastable};

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
