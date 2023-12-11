//! Generic Artifact abstraction for Wasmer Engines.

mod artifact_builder;
mod trampoline;

pub use self::artifact_builder::{ArtifactBuild, ArtifactBuildFromArchive, ModuleFromArchive};
pub use self::trampoline::*;
