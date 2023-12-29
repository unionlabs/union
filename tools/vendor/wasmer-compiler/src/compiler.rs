//! This module mainly outputs the `Compiler` trait that custom
//! compilers will need to implement.

use crate::lib::std::boxed::Box;
use crate::lib::std::sync::Arc;
use crate::translator::ModuleMiddleware;
use crate::FunctionBodyData;
use crate::ModuleTranslationState;
use enumset::EnumSet;
use wasmer_types::compilation::function::Compilation;
use wasmer_types::compilation::module::CompileModuleInfo;
use wasmer_types::compilation::symbols::SymbolRegistry;
use wasmer_types::compilation::target::Target;
use wasmer_types::entity::PrimaryMap;
use wasmer_types::error::CompileError;
use wasmer_types::{CpuFeature, Features, LocalFunctionIndex};
use wasmparser::{Validator, WasmFeatures};

/// The compiler configuration options.
pub trait CompilerConfig {
    /// Enable Position Independent Code (PIC).
    ///
    /// This is required for shared object generation (Native Engine),
    /// but will make the JIT Engine to fail, since PIC is not yet
    /// supported in the JIT linking phase.
    fn enable_pic(&mut self) {
        // By default we do nothing, each backend will need to customize this
        // in case they do something special for emitting PIC code.
    }

    /// Enable compiler IR verification.
    ///
    /// For compilers capable of doing so, this enables internal consistency
    /// checking.
    fn enable_verifier(&mut self) {
        // By default we do nothing, each backend will need to customize this
        // in case they create an IR that they can verify.
    }

    /// Enable NaN canonicalization.
    ///
    /// NaN canonicalization is useful when trying to run WebAssembly
    /// deterministically across different architectures.
    fn canonicalize_nans(&mut self, _enable: bool) {
        // By default we do nothing, each backend will need to customize this
        // in case they create an IR that they can verify.
    }

    /// Gets the custom compiler config
    fn compiler(self: Box<Self>) -> Box<dyn Compiler>;

    /// Gets the default features for this compiler in the given target
    fn default_features_for_target(&self, _target: &Target) -> Features {
        Features::default()
    }

    /// Pushes a middleware onto the back of the middleware chain.
    fn push_middleware(&mut self, middleware: Arc<dyn ModuleMiddleware>);
}

impl<T> From<T> for Box<dyn CompilerConfig + 'static>
where
    T: CompilerConfig + 'static,
{
    fn from(other: T) -> Self {
        Box::new(other)
    }
}

/// An implementation of a Compiler from parsed WebAssembly module to Compiled native code.
pub trait Compiler: Send {
    /// Returns a descriptive name for this compiler.
    ///
    /// Note that this is an API breaking change since 3.0
    fn name(&self) -> &str;

    /// Validates a module.
    ///
    /// It returns the a succesful Result in case is valid, `CompileError` in case is not.
    fn validate_module(&self, features: &Features, data: &[u8]) -> Result<(), CompileError> {
        let wasm_features = WasmFeatures {
            bulk_memory: features.bulk_memory,
            threads: features.threads,
            reference_types: features.reference_types,
            multi_value: features.multi_value,
            simd: features.simd,
            tail_call: features.tail_call,
            multi_memory: features.multi_memory,
            memory64: features.memory64,
            exceptions: features.exceptions,
            deterministic_only: false,
            extended_const: features.extended_const,
            relaxed_simd: features.relaxed_simd,
            mutable_global: true,
            saturating_float_to_int: true,
            sign_extension: true,
            component_model: false,
        };
        let mut validator = Validator::new_with_features(wasm_features);
        validator
            .validate_all(data)
            .map_err(|e| CompileError::Validate(format!("{}", e)))?;
        Ok(())
    }

    /// Compiles a parsed module.
    ///
    /// It returns the [`Compilation`] or a [`CompileError`].
    fn compile_module(
        &self,
        target: &Target,
        module: &CompileModuleInfo,
        module_translation: &ModuleTranslationState,
        // The list of function bodies
        function_body_inputs: PrimaryMap<LocalFunctionIndex, FunctionBodyData<'_>>,
    ) -> Result<Compilation, CompileError>;

    /// Compiles a module into a native object file.
    ///
    /// It returns the bytes as a `&[u8]` or a [`CompileError`].
    fn experimental_native_compile_module(
        &self,
        _target: &Target,
        _module: &CompileModuleInfo,
        _module_translation: &ModuleTranslationState,
        // The list of function bodies
        _function_body_inputs: &PrimaryMap<LocalFunctionIndex, FunctionBodyData<'_>>,
        _symbol_registry: &dyn SymbolRegistry,
        // The metadata to inject into the wasmer_metadata section of the object file.
        _wasmer_metadata: &[u8],
    ) -> Option<Result<Vec<u8>, CompileError>> {
        None
    }

    /// Get the middlewares for this compiler
    fn get_middlewares(&self) -> &[Arc<dyn ModuleMiddleware>];

    /// Get the CpuFeatues used by the compiler
    fn get_cpu_features_used(&self, cpu_features: &EnumSet<CpuFeature>) -> EnumSet<CpuFeature> {
        *cpu_features
    }
}
