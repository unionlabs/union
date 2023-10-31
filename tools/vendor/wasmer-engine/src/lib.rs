//! Generic Engine abstraction for Wasmer Engines.

#![deny(missing_docs, trivial_numeric_casts, unused_extern_crates)]
#![warn(unused_import_braces)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        clippy::new_without_default,
        clippy::upper_case_acronyms,
        clippy::new_without_default
    )
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
mod engine;
mod error;
mod export;
mod resolver;
mod trap;
mod tunables;

pub use crate::artifact::Artifact;
pub use crate::engine::{Engine, EngineId};
pub use crate::error::{InstantiationError, LinkError};
pub use crate::export::{Export, ExportFunction, ExportFunctionMetadata};
pub use crate::resolver::{
    resolve_imports, ChainableNamedResolver, NamedResolver, NamedResolverChain, NullResolver,
    Resolver,
};
pub use crate::trap::*;
pub use crate::tunables::Tunables;
pub use wasmer_artifact::{ArtifactCreate, MetadataHeader};
pub use wasmer_artifact::{DeserializeError, ImportError, SerializeError};

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
