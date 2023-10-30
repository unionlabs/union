//! Define `UniversalArtifactBuild` to allow compiling and instantiating to be
//! done as separate steps.

use crate::serialize::SerializableCompilation;
use crate::serialize::SerializableModule;
#[cfg(feature = "compiler")]
use crate::trampoline::{libcall_trampoline_len, make_libcall_trampolines};
use crate::{ArtifactCreate, UniversalEngineBuilder};
use enumset::EnumSet;
use loupe::MemoryUsage;
use std::mem;
use std::sync::Arc;
use wasmer_artifact::{MetadataHeader, SerializeError};
use wasmer_compiler::{
    CompileError, CompileModuleInfo, CompiledFunctionFrameInfo, CpuFeature, CustomSection, Dwarf,
    Features, FunctionBody, ModuleEnvironment, ModuleMiddlewareChain, Relocation, SectionIndex,
    Target, Triple,
};
use wasmer_types::entity::PrimaryMap;
use wasmer_types::{
    FunctionIndex, LocalFunctionIndex, MemoryIndex, MemoryStyle, ModuleInfo, OwnedDataInitializer,
    SignatureIndex, TableIndex, TableStyle,
};

/// A compiled wasm module, ready to be instantiated.
#[derive(MemoryUsage)]
pub struct UniversalArtifactBuild {
    serializable: SerializableModule,
}

impl UniversalArtifactBuild {
    /// Header signature for wasmu binary
    pub const MAGIC_HEADER: &'static [u8; 16] = b"wasmer-universal";

    /// Check if the provided bytes look like a serialized `UniversalArtifactBuild`.
    pub fn is_deserializable(bytes: &[u8]) -> bool {
        bytes.starts_with(Self::MAGIC_HEADER)
    }

    /// Compile a data buffer into a `UniversalArtifactBuild`, which may then be instantiated.
    #[cfg(feature = "compiler")]
    pub fn new(
        inner_engine: &mut UniversalEngineBuilder,
        data: &[u8],
        target: &Target,
        memory_styles: PrimaryMap<MemoryIndex, MemoryStyle>,
        table_styles: PrimaryMap<TableIndex, TableStyle>,
    ) -> Result<Self, CompileError> {
        let environ = ModuleEnvironment::new();
        let features = inner_engine.features();

        let translation = environ.translate(data).map_err(CompileError::Wasm)?;

        let compiler = inner_engine.compiler()?;

        // We try to apply the middleware first
        let mut module = translation.module;
        let middlewares = compiler.get_middlewares();
        middlewares.apply_on_module_info(&mut module);

        let compile_info = CompileModuleInfo {
            module: Arc::new(module),
            features: features.clone(),
            memory_styles,
            table_styles,
        };

        // Compile the Module
        let compilation = compiler.compile_module(
            target,
            &compile_info,
            // SAFETY: Calling `unwrap` is correct since
            // `environ.translate()` above will write some data into
            // `module_translation_state`.
            translation.module_translation_state.as_ref().unwrap(),
            translation.function_body_inputs,
        )?;
        let function_call_trampolines = compilation.get_function_call_trampolines();
        let dynamic_function_trampolines = compilation.get_dynamic_function_trampolines();

        let data_initializers = translation
            .data_initializers
            .iter()
            .map(OwnedDataInitializer::new)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let frame_infos = compilation.get_frame_info();

        // Synthesize a custom section to hold the libcall trampolines.
        let mut custom_sections = compilation.get_custom_sections();
        let mut custom_section_relocations = compilation.get_custom_section_relocations();
        let libcall_trampolines_section = make_libcall_trampolines(target);
        custom_section_relocations.push(libcall_trampolines_section.relocations.clone());
        let libcall_trampolines = custom_sections.push(libcall_trampolines_section);
        let libcall_trampoline_len = libcall_trampoline_len(target) as u32;

        let serializable_compilation = SerializableCompilation {
            function_bodies: compilation.get_function_bodies(),
            function_relocations: compilation.get_relocations(),
            function_frame_info: frame_infos,
            function_call_trampolines,
            dynamic_function_trampolines,
            custom_sections,
            custom_section_relocations,
            debug: compilation.get_debug(),
            libcall_trampolines,
            libcall_trampoline_len,
        };
        let serializable = SerializableModule {
            compilation: serializable_compilation,
            compile_info,
            data_initializers,
            cpu_features: target.cpu_features().as_u64(),
        };
        Ok(Self { serializable })
    }

    /// Compile a data buffer into a `UniversalArtifactBuild`, which may then be instantiated.
    #[cfg(not(feature = "compiler"))]
    pub fn new(_engine: &UniversalEngineBuilder, _data: &[u8]) -> Result<Self, CompileError> {
        Err(CompileError::Codegen(
            "Compilation is not enabled in the engine".to_string(),
        ))
    }

    /// Create a new UniversalArtifactBuild from a SerializableModule
    pub fn from_serializable(serializable: SerializableModule) -> Self {
        Self { serializable }
    }

    /// Get the default extension when serializing this artifact
    pub fn get_default_extension(_triple: &Triple) -> &'static str {
        // `.wasmu` is the default extension for all the triples. It
        // stands for “Wasm Universal”.
        "wasmu"
    }

    /// Get Functions Bodies ref
    pub fn get_function_bodies_ref(&self) -> &PrimaryMap<LocalFunctionIndex, FunctionBody> {
        &self.serializable.compilation.function_bodies
    }

    /// Get Functions Call Trampolines ref
    pub fn get_function_call_trampolines_ref(&self) -> &PrimaryMap<SignatureIndex, FunctionBody> {
        &self.serializable.compilation.function_call_trampolines
    }

    /// Get Dynamic Functions Call Trampolines ref
    pub fn get_dynamic_function_trampolines_ref(&self) -> &PrimaryMap<FunctionIndex, FunctionBody> {
        &self.serializable.compilation.dynamic_function_trampolines
    }

    /// Get Custom Sections ref
    pub fn get_custom_sections_ref(&self) -> &PrimaryMap<SectionIndex, CustomSection> {
        &self.serializable.compilation.custom_sections
    }

    /// Get Function Relocations
    pub fn get_function_relocations(&self) -> PrimaryMap<LocalFunctionIndex, Vec<Relocation>> {
        self.serializable.compilation.function_relocations.clone()
    }

    /// Get Function Relocations ref
    pub fn get_custom_section_relocations_ref(&self) -> &PrimaryMap<SectionIndex, Vec<Relocation>> {
        &self.serializable.compilation.custom_section_relocations
    }

    /// Get LibCall Trampoline Section Index
    pub fn get_libcall_trampolines(&self) -> SectionIndex {
        self.serializable.compilation.libcall_trampolines
    }

    /// Get LibCall Trampoline Length
    pub fn get_libcall_trampoline_len(&self) -> usize {
        self.serializable.compilation.libcall_trampoline_len as usize
    }

    /// Get Debug optional Dwarf ref
    pub fn get_debug_ref(&self) -> &Option<Dwarf> {
        &self.serializable.compilation.debug
    }

    /// Get Function Relocations ref
    pub fn get_frame_info_ref(&self) -> &PrimaryMap<LocalFunctionIndex, CompiledFunctionFrameInfo> {
        &self.serializable.compilation.function_frame_info
    }
}

impl ArtifactCreate for UniversalArtifactBuild {
    fn module(&self) -> Arc<ModuleInfo> {
        self.serializable.compile_info.module.clone()
    }

    fn module_ref(&self) -> &ModuleInfo {
        &self.serializable.compile_info.module
    }

    fn module_mut(&mut self) -> Option<&mut ModuleInfo> {
        Arc::get_mut(&mut self.serializable.compile_info.module)
    }

    fn features(&self) -> &Features {
        &self.serializable.compile_info.features
    }

    fn cpu_features(&self) -> EnumSet<CpuFeature> {
        EnumSet::from_u64(self.serializable.cpu_features)
    }

    fn data_initializers(&self) -> &[OwnedDataInitializer] {
        &*self.serializable.data_initializers
    }

    fn memory_styles(&self) -> &PrimaryMap<MemoryIndex, MemoryStyle> {
        &self.serializable.compile_info.memory_styles
    }

    fn table_styles(&self) -> &PrimaryMap<TableIndex, TableStyle> {
        &self.serializable.compile_info.table_styles
    }

    fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        let serialized_data = self.serializable.serialize()?;
        assert!(mem::align_of::<SerializableModule>() <= MetadataHeader::ALIGN);

        let mut metadata_binary = vec![];
        metadata_binary.extend(Self::MAGIC_HEADER);
        metadata_binary.extend(MetadataHeader::new(serialized_data.len()));
        metadata_binary.extend(serialized_data);
        Ok(metadata_binary)
    }
}
