use std::{path::Path, sync::Arc};

use shared_buffer::OwnedBuffer;
pub use wasmer_compiler::{
    Artifact, BaseTunables, CompilerConfig, Engine, EngineBuilder, Tunables,
};
#[cfg(feature = "compiler")]
use wasmer_types::Features;
use wasmer_types::{DeserializeError, Target};

/// Returns the default engine for the Sys engine
pub(crate) fn default_engine() -> Engine {
    // We store them on a function that returns to make
    // sure this function doesn't emit a compile error even if
    // more than one compiler is enabled.
    #[allow(unreachable_code)]
    #[cfg(any(feature = "cranelift", feature = "llvm", feature = "singlepass"))]
    fn get_config() -> impl wasmer_compiler::CompilerConfig + 'static {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cranelift")] {
                wasmer_compiler_cranelift::Cranelift::default()
            } else if #[cfg(feature = "llvm")] {
                wasmer_compiler_llvm::LLVM::default()
            } else if #[cfg(feature = "singlepass")] {
                wasmer_compiler_singlepass::Singlepass::default()
            } else {
                compile_error!("No default compiler chosen")
            }
        }
    }

    #[allow(unreachable_code, unused_mut)]
    fn get_engine() -> Engine {
        cfg_if::cfg_if! {
            if #[cfg(feature = "compiler")] {
                cfg_if::cfg_if! {
                    if #[cfg(any(feature = "cranelift", feature = "llvm", feature = "singlepass"))]
                    {
                        let config = get_config();
                        EngineBuilder::new(Box::new(config) as Box<dyn wasmer_compiler::CompilerConfig>)
                            .engine()
                    } else {
                        EngineBuilder::headless()
                            .engine()
                    }
                }
            } else {
                EngineBuilder::headless().engine()
            }
        }
    }

    let mut engine = get_engine();
    let tunables = BaseTunables::for_target(engine.target());
    engine.set_tunables(tunables);
    engine
}

/// The custom trait to access to all the `sys` function in the common
/// engine.
pub trait NativeEngineExt {
    /// Create a new `Engine` with the given config
    #[cfg(feature = "compiler")]
    fn new(compiler_config: Box<dyn CompilerConfig>, target: Target, features: Features) -> Self;

    /// Create a headless `Engine`
    ///
    /// A headless engine is an engine without any compiler attached.
    /// This is useful for assuring a minimal runtime for running
    /// WebAssembly modules.
    ///
    /// For example, for running in IoT devices where compilers are very
    /// expensive, or also to optimize startup speed.
    ///
    /// # Important
    ///
    /// Headless engines can't compile or validate any modules,
    /// they just take already processed Modules (via `Module::serialize`).
    fn headless() -> Self;

    /// Gets the target
    fn target(&self) -> &Target;

    /// Attach a Tunable to this engine
    fn set_tunables(&mut self, tunables: impl Tunables + Send + Sync + 'static);

    /// Get a reference to attached Tunable of this engine
    fn tunables(&self) -> &dyn Tunables;

    /// Load a serialized WebAssembly module from a memory mapped file and deserialize it.
    ///
    /// NOTE: you should almost always prefer [`Self::deserialize_from_mmapped_file`].
    ///
    /// # Safety
    /// See [`Artifact::deserialize_unchecked`].
    unsafe fn deserialize_from_mmapped_file_unchecked(
        &self,
        file_ref: &Path,
    ) -> Result<crate::Module, DeserializeError>;

    /// Load a serialized WebAssembly module from a memory mapped file and deserialize it.
    ///
    /// # Safety
    /// See [`Artifact::deserialize`].
    unsafe fn deserialize_from_mmapped_file(
        &self,
        file_ref: &Path,
    ) -> Result<crate::Module, DeserializeError>;
}

impl NativeEngineExt for crate::engine::Engine {
    #[cfg(feature = "compiler")]
    fn new(compiler_config: Box<dyn CompilerConfig>, target: Target, features: Features) -> Self {
        Self(Engine::new(compiler_config, target, features))
    }

    fn headless() -> Self {
        Self(Engine::headless())
    }

    fn target(&self) -> &Target {
        self.0.target()
    }

    fn set_tunables(&mut self, tunables: impl Tunables + Send + Sync + 'static) {
        self.0.set_tunables(tunables)
    }

    fn tunables(&self) -> &dyn Tunables {
        self.0.tunables()
    }

    unsafe fn deserialize_from_mmapped_file_unchecked(
        &self,
        file_ref: &Path,
    ) -> Result<crate::Module, DeserializeError> {
        let bytes = std::fs::read(file_ref)?;
        let artifact = Arc::new(Artifact::deserialize_unchecked(&self.0, bytes.into())?);
        Ok(crate::Module(super::module::Module::from_artifact(
            artifact,
        )))
    }

    unsafe fn deserialize_from_mmapped_file(
        &self,
        file_ref: &Path,
    ) -> Result<crate::Module, DeserializeError> {
        let file = std::fs::File::open(file_ref)?;
        let artifact = Arc::new(Artifact::deserialize(
            &self.0,
            OwnedBuffer::from_file(&file)
                .map_err(|e| DeserializeError::Generic(format!("{e:?}")))?,
        )?);
        Ok(crate::Module(super::module::Module::from_artifact(
            artifact,
        )))
    }
}
