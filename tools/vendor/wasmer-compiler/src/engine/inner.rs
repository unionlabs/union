//! Universal compilation.

use crate::engine::builder::EngineBuilder;
#[cfg(not(target_arch = "wasm32"))]
use crate::Artifact;
#[cfg(not(target_arch = "wasm32"))]
use crate::BaseTunables;
#[cfg(not(target_arch = "wasm32"))]
use crate::CodeMemory;
#[cfg(not(target_arch = "wasm32"))]
use crate::GlobalFrameInfoRegistration;
#[cfg(feature = "compiler")]
use crate::{Compiler, CompilerConfig};
#[cfg(not(target_arch = "wasm32"))]
use crate::{FunctionExtent, Tunables};
#[cfg(not(target_arch = "wasm32"))]
use shared_buffer::OwnedBuffer;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use wasmer_types::{
    entity::PrimaryMap, DeserializeError, FunctionBodyLike, FunctionIndex, FunctionType,
    LocalFunctionIndex, SignatureIndex,
};
use wasmer_types::{CompileError, Features, ModuleInfo, Target};
#[cfg(not(target_arch = "wasm32"))]
use wasmer_types::{CustomSectionLike, CustomSectionProtection, SectionIndex};
#[cfg(not(target_arch = "wasm32"))]
use wasmer_vm::{
    FunctionBodyPtr, SectionBodyPtr, SignatureRegistry, VMFunctionBody, VMSharedSignatureIndex,
    VMTrampoline,
};

/// A WebAssembly `Universal` Engine.
#[derive(Clone)]
pub struct Engine {
    inner: Arc<Mutex<EngineInner>>,
    /// The target for the compiler
    target: Arc<Target>,
    engine_id: EngineId,
    #[cfg(not(target_arch = "wasm32"))]
    tunables: Arc<dyn Tunables + Send + Sync>,
    name: String,
}

impl Engine {
    /// Create a new `Engine` with the given config
    #[cfg(feature = "compiler")]
    pub fn new(
        compiler_config: Box<dyn CompilerConfig>,
        target: Target,
        features: Features,
    ) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let tunables = BaseTunables::for_target(&target);
        let compiler = compiler_config.compiler();
        let name = format!("engine-{}", compiler.name());
        Self {
            inner: Arc::new(Mutex::new(EngineInner {
                compiler: Some(compiler),
                features,
                #[cfg(not(target_arch = "wasm32"))]
                code_memory: vec![],
                #[cfg(not(target_arch = "wasm32"))]
                signatures: SignatureRegistry::new(),
            })),
            target: Arc::new(target),
            engine_id: EngineId::default(),
            #[cfg(not(target_arch = "wasm32"))]
            tunables: Arc::new(tunables),
            name,
        }
    }

    #[cfg(not(feature = "compiler"))]
    pub fn new(
        compiler_config: Box<dyn CompilerConfig>,
        target: Target,
        features: Features,
    ) -> Self {
        panic!("The engine is not compiled with any compiler support")
    }

    /// Returns the name of this engine
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the deterministic id of this engine
    pub fn deterministic_id(&self) -> &str {
        // TODO: add a `deterministic_id` to the Compiler, so two
        // compilers can actually serialize into a different deterministic_id
        // if their configuration is different (eg. LLVM with optimizations vs LLVM
        // without optimizations)
        self.name.as_str()
    }

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
    pub fn headless() -> Self {
        let target = Target::default();
        #[cfg(not(target_arch = "wasm32"))]
        let tunables = BaseTunables::for_target(&target);
        Self {
            inner: Arc::new(Mutex::new(EngineInner {
                #[cfg(feature = "compiler")]
                compiler: None,
                #[cfg(feature = "compiler")]
                features: Features::default(),
                #[cfg(not(target_arch = "wasm32"))]
                code_memory: vec![],
                #[cfg(not(target_arch = "wasm32"))]
                signatures: SignatureRegistry::new(),
            })),
            target: Arc::new(target),
            engine_id: EngineId::default(),
            #[cfg(not(target_arch = "wasm32"))]
            tunables: Arc::new(tunables),
            name: "engine-headless".to_string(),
        }
    }

    /// Get reference to `EngineInner`.
    pub fn inner(&self) -> std::sync::MutexGuard<'_, EngineInner> {
        self.inner.lock().unwrap()
    }

    /// Get mutable reference to `EngineInner`.
    pub fn inner_mut(&self) -> std::sync::MutexGuard<'_, EngineInner> {
        self.inner.lock().unwrap()
    }

    /// Gets the target
    pub fn target(&self) -> &Target {
        &self.target
    }

    /// Register a signature
    #[cfg(not(target_arch = "wasm32"))]
    pub fn register_signature(&self, func_type: &FunctionType) -> VMSharedSignatureIndex {
        let compiler = self.inner();
        compiler.signatures().register(func_type)
    }

    /// Lookup a signature
    #[cfg(not(target_arch = "wasm32"))]
    pub fn lookup_signature(&self, sig: VMSharedSignatureIndex) -> Option<FunctionType> {
        let compiler = self.inner();
        compiler.signatures().lookup(sig)
    }

    /// Validates a WebAssembly module
    #[cfg(feature = "compiler")]
    pub fn validate(&self, binary: &[u8]) -> Result<(), CompileError> {
        self.inner().validate(binary)
    }

    /// Compile a WebAssembly binary
    #[cfg(feature = "compiler")]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn compile(&self, binary: &[u8]) -> Result<Arc<Artifact>, CompileError> {
        Ok(Arc::new(Artifact::new(
            self,
            binary,
            self.tunables.as_ref(),
        )?))
    }

    /// Compile a WebAssembly binary
    #[cfg(not(feature = "compiler"))]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn compile(
        &self,
        _binary: &[u8],
        _tunables: &dyn Tunables,
    ) -> Result<Arc<Artifact>, CompileError> {
        Err(CompileError::Codegen(
            "The Engine is operating in headless mode, so it can not compile Modules.".to_string(),
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Deserializes a WebAssembly module which was previously serialized with
    /// [`Module::serialize`].
    ///
    /// # Safety
    ///
    /// See [`Artifact::deserialize_unchecked`].
    pub unsafe fn deserialize_unchecked(
        &self,
        bytes: OwnedBuffer,
    ) -> Result<Arc<Artifact>, DeserializeError> {
        Ok(Arc::new(Artifact::deserialize_unchecked(self, bytes)?))
    }

    /// Deserializes a WebAssembly module which was previously serialized with
    /// [`Module::serialize`].
    ///
    /// # Safety
    ///
    /// See [`Artifact::deserialize`].
    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn deserialize(
        &self,
        bytes: OwnedBuffer,
    ) -> Result<Arc<Artifact>, DeserializeError> {
        Ok(Arc::new(Artifact::deserialize(self, bytes)?))
    }

    /// Deserializes a WebAssembly module from a path.
    ///
    /// # Safety
    /// See [`Artifact::deserialize`].
    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn deserialize_from_file(
        &self,
        file_ref: &Path,
    ) -> Result<Arc<Artifact>, DeserializeError> {
        let file = std::fs::File::open(file_ref)?;
        self.deserialize(
            OwnedBuffer::from_file(&file).map_err(|e| DeserializeError::Generic(e.to_string()))?,
        )
    }

    /// Deserialize from a file path.
    ///
    /// # Safety
    ///
    /// See [`Artifact::deserialize_unchecked`].
    #[cfg(not(target_arch = "wasm32"))]
    pub unsafe fn deserialize_from_file_unchecked(
        &self,
        file_ref: &Path,
    ) -> Result<Arc<Artifact>, DeserializeError> {
        let file = std::fs::File::open(file_ref)?;
        self.deserialize_unchecked(
            OwnedBuffer::from_file(&file).map_err(|e| DeserializeError::Generic(e.to_string()))?,
        )
    }

    /// A unique identifier for this object.
    ///
    /// This exists to allow us to compare two Engines for equality. Otherwise,
    /// comparing two trait objects unsafely relies on implementation details
    /// of trait representation.
    pub fn id(&self) -> &EngineId {
        &self.engine_id
    }

    /// Clone the engine
    pub fn cloned(&self) -> Self {
        self.clone()
    }

    /// Attach a Tunable to this engine
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_tunables(&mut self, tunables: impl Tunables + Send + Sync + 'static) {
        self.tunables = Arc::new(tunables);
    }

    /// Get a reference to attached Tunable of this engine
    #[cfg(not(target_arch = "wasm32"))]
    pub fn tunables(&self) -> &dyn Tunables {
        self.tunables.as_ref()
    }
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Engine")
            .field("target", &self.target)
            .field("engine_id", &self.engine_id)
            .field("name", &self.name)
            .finish()
    }
}

/// The inner contents of `Engine`
pub struct EngineInner {
    #[cfg(feature = "compiler")]
    /// The compiler and cpu features
    compiler: Option<Box<dyn Compiler>>,
    #[cfg(feature = "compiler")]
    /// The compiler and cpu features
    features: Features,
    /// The code memory is responsible of publishing the compiled
    /// functions to memory.
    #[cfg(not(target_arch = "wasm32"))]
    code_memory: Vec<CodeMemory>,
    /// The signature registry is used mainly to operate with trampolines
    /// performantly.
    #[cfg(not(target_arch = "wasm32"))]
    signatures: SignatureRegistry,
}

impl EngineInner {
    /// Gets the compiler associated to this engine.
    #[cfg(feature = "compiler")]
    pub fn compiler(&self) -> Result<&dyn Compiler, CompileError> {
        match self.compiler.as_ref() {
            None => Err(CompileError::Codegen(
                "No compiler compiled into executable".to_string(),
            )),
            Some(compiler) => Ok(&**compiler),
        }
    }

    /// Validate the module
    #[cfg(feature = "compiler")]
    pub fn validate(&self, data: &[u8]) -> Result<(), CompileError> {
        let compiler = self.compiler()?;
        compiler.validate_module(&self.features, data)
    }

    /// The Wasm features
    #[cfg(feature = "compiler")]
    pub fn features(&self) -> &Features {
        &self.features
    }

    /// Allocate compiled functions into memory
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::type_complexity)]
    pub(crate) fn allocate<'a, FunctionBody, CustomSection>(
        &'a mut self,
        _module: &ModuleInfo,
        functions: impl ExactSizeIterator<Item = &'a FunctionBody> + 'a,
        function_call_trampolines: impl ExactSizeIterator<Item = &'a FunctionBody> + 'a,
        dynamic_function_trampolines: impl ExactSizeIterator<Item = &'a FunctionBody> + 'a,
        custom_sections: impl ExactSizeIterator<Item = &'a CustomSection> + Clone + 'a,
    ) -> Result<
        (
            PrimaryMap<LocalFunctionIndex, FunctionExtent>,
            PrimaryMap<SignatureIndex, VMTrampoline>,
            PrimaryMap<FunctionIndex, FunctionBodyPtr>,
            PrimaryMap<SectionIndex, SectionBodyPtr>,
        ),
        CompileError,
    >
    where
        FunctionBody: FunctionBodyLike<'a> + 'a,
        CustomSection: CustomSectionLike<'a> + 'a,
    {
        let functions_len = functions.len();
        let function_call_trampolines_len = function_call_trampolines.len();

        let function_bodies = functions
            .chain(function_call_trampolines)
            .chain(dynamic_function_trampolines)
            .collect::<Vec<_>>();
        let (executable_sections, data_sections): (Vec<_>, _) = custom_sections
            .clone()
            .partition(|section| *section.protection() == CustomSectionProtection::ReadExecute);
        self.code_memory.push(CodeMemory::new());

        let (mut allocated_functions, allocated_executable_sections, allocated_data_sections) =
            self.code_memory
                .last_mut()
                .unwrap()
                .allocate(
                    function_bodies.as_slice(),
                    executable_sections.as_slice(),
                    data_sections.as_slice(),
                )
                .map_err(|message| {
                    CompileError::Resource(format!(
                        "failed to allocate memory for functions: {}",
                        message
                    ))
                })?;

        let allocated_functions_result = allocated_functions
            .drain(0..functions_len)
            .map(|slice| FunctionExtent {
                ptr: FunctionBodyPtr(slice.as_ptr()),
                length: slice.len(),
            })
            .collect::<PrimaryMap<LocalFunctionIndex, _>>();

        let mut allocated_function_call_trampolines: PrimaryMap<SignatureIndex, VMTrampoline> =
            PrimaryMap::new();
        for ptr in allocated_functions
            .drain(0..function_call_trampolines_len)
            .map(|slice| slice.as_ptr())
        {
            let trampoline =
                unsafe { std::mem::transmute::<*const VMFunctionBody, VMTrampoline>(ptr) };
            allocated_function_call_trampolines.push(trampoline);
        }

        let allocated_dynamic_function_trampolines = allocated_functions
            .drain(..)
            .map(|slice| FunctionBodyPtr(slice.as_ptr()))
            .collect::<PrimaryMap<FunctionIndex, _>>();

        let mut exec_iter = allocated_executable_sections.iter();
        let mut data_iter = allocated_data_sections.iter();
        let allocated_custom_sections = custom_sections
            .map(|section| {
                SectionBodyPtr(
                    if *section.protection() == CustomSectionProtection::ReadExecute {
                        exec_iter.next()
                    } else {
                        data_iter.next()
                    }
                    .unwrap()
                    .as_ptr(),
                )
            })
            .collect::<PrimaryMap<SectionIndex, _>>();

        Ok((
            allocated_functions_result,
            allocated_function_call_trampolines,
            allocated_dynamic_function_trampolines,
            allocated_custom_sections,
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Make memory containing compiled code executable.
    pub(crate) fn publish_compiled_code(&mut self) {
        self.code_memory.last_mut().unwrap().publish();
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Register DWARF-type exception handling information associated with the code.
    pub(crate) fn publish_eh_frame(&mut self, eh_frame: Option<&[u8]>) -> Result<(), CompileError> {
        self.code_memory
            .last_mut()
            .unwrap()
            .unwind_registry_mut()
            .publish(eh_frame)
            .map_err(|e| {
                CompileError::Resource(format!("Error while publishing the unwind code: {}", e))
            })?;
        Ok(())
    }

    /// Shared signature registry.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn signatures(&self) -> &SignatureRegistry {
        &self.signatures
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Register the frame info for the code memory
    pub(crate) fn register_frame_info(&mut self, frame_info: GlobalFrameInfoRegistration) {
        self.code_memory
            .last_mut()
            .unwrap()
            .register_frame_info(frame_info);
    }
}

#[cfg(feature = "compiler")]
impl From<Box<dyn CompilerConfig>> for Engine {
    fn from(config: Box<dyn CompilerConfig>) -> Self {
        EngineBuilder::new(config).engine()
    }
}

impl From<EngineBuilder> for Engine {
    fn from(engine_builder: EngineBuilder) -> Self {
        engine_builder.engine()
    }
}

impl From<&Self> for Engine {
    fn from(engine_ref: &Self) -> Self {
        engine_ref.cloned()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
/// A unique identifier for an Engine.
pub struct EngineId {
    id: usize,
}

impl EngineId {
    /// Format this identifier as a string.
    pub fn id(&self) -> String {
        format!("{}", &self.id)
    }
}

impl Clone for EngineId {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Default for EngineId {
    fn default() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self {
            id: NEXT_ID.fetch_add(1, SeqCst),
        }
    }
}
