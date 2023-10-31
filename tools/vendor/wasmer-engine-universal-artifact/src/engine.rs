//! Universal compilation.

use loupe::MemoryUsage;
use wasmer_compiler::CompileError;
use wasmer_compiler::Compiler;
use wasmer_types::Features;

/// The Builder contents of `UniversalEngine`
#[derive(MemoryUsage)]
pub struct UniversalEngineBuilder {
    /// The compiler
    #[cfg(feature = "compiler")]
    compiler: Option<Box<dyn Compiler>>,
    /// The features to compile the Wasm module with
    features: Features,
}

impl UniversalEngineBuilder {
    /// Create a new builder with pre-made components
    #[cfg(feature = "compiler")]
    pub fn new(compiler: Option<Box<dyn Compiler>>, features: Features) -> Self {
        UniversalEngineBuilder { compiler, features }
    }

    /// Gets the compiler associated to this engine.
    #[cfg(feature = "compiler")]
    pub fn compiler(&self) -> Result<&dyn Compiler, CompileError> {
        if self.compiler.is_none() {
            return Err(CompileError::Codegen(
                "The UniversalEngine is not compiled in.".to_string(),
            ));
        }
        Ok(&**self.compiler.as_ref().unwrap())
    }

    /// Gets the compiler associated to this engine.
    #[cfg(not(feature = "compiler"))]
    pub fn compiler(&self) -> Result<&dyn Compiler, CompileError> {
        return Err(CompileError::Codegen(
            "The UniversalEngine is not compiled in.".to_string(),
        ));
    }

    /// Validate the module
    #[cfg(feature = "compiler")]
    pub fn validate<'data>(&self, data: &'data [u8]) -> Result<(), CompileError> {
        self.compiler()?.validate_module(self.features(), data)
    }

    /// Validate the module
    #[cfg(not(feature = "compiler"))]
    pub fn validate<'data>(&self, _data: &'data [u8]) -> Result<(), CompileError> {
        Err(CompileError::Validate(
            "The UniversalEngine is not compiled with compiler support, which is required for validating"
                .to_string(),
        ))
    }

    /// The Wasm features
    pub fn features(&self) -> &Features {
        &self.features
    }
}
