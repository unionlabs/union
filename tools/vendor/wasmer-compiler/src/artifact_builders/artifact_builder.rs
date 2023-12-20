//! Define `ArtifactBuild` to allow compiling and instantiating to be
//! done as separate steps.

#[cfg(feature = "compiler")]
use super::trampoline::{libcall_trampoline_len, make_libcall_trampolines};
use crate::ArtifactCreate;
use crate::EngineInner;
use crate::Features;
use crate::{ModuleEnvironment, ModuleMiddlewareChain};
use core::mem::MaybeUninit;
use enumset::EnumSet;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::option::ArchivedOption;
use self_cell::self_cell;
use shared_buffer::OwnedBuffer;
use std::sync::Arc;
use wasmer_types::entity::{ArchivedPrimaryMap, PrimaryMap};
use wasmer_types::ArchivedOwnedDataInitializer;
use wasmer_types::ArchivedSerializableCompilation;
use wasmer_types::ArchivedSerializableModule;
#[cfg(feature = "compiler")]
use wasmer_types::CompileModuleInfo;
use wasmer_types::DeserializeError;
use wasmer_types::{
    CompileError, CpuFeature, CustomSection, Dwarf, FunctionIndex, LocalFunctionIndex, MemoryIndex,
    MemoryStyle, ModuleInfo, OwnedDataInitializer, Relocation, SectionIndex, SignatureIndex,
    TableIndex, TableStyle, Target,
};
use wasmer_types::{
    CompiledFunctionFrameInfo, FunctionBody, SerializableCompilation, SerializableModule,
};
use wasmer_types::{MetadataHeader, SerializeError};

/// A compiled wasm module, ready to be instantiated.
pub struct ArtifactBuild {
    serializable: SerializableModule,
}

impl ArtifactBuild {
    /// Header signature for wasmu binary
    pub const MAGIC_HEADER: &'static [u8; 16] = b"wasmer-universal";

    /// Check if the provided bytes look like a serialized `ArtifactBuild`.
    pub fn is_deserializable(bytes: &[u8]) -> bool {
        bytes.starts_with(Self::MAGIC_HEADER)
    }

    /// Compile a data buffer into a `ArtifactBuild`, which may then be instantiated.
    #[cfg(feature = "compiler")]
    pub fn new(
        inner_engine: &mut EngineInner,
        data: &[u8],
        target: &Target,
        memory_styles: PrimaryMap<MemoryIndex, MemoryStyle>,
        table_styles: PrimaryMap<TableIndex, TableStyle>,
    ) -> Result<Self, CompileError> {
        let environ = ModuleEnvironment::new();
        let features = inner_engine.features().clone();

        let translation = environ.translate(data).map_err(CompileError::Wasm)?;

        let compiler = inner_engine.compiler()?;

        // We try to apply the middleware first
        let mut module = translation.module;
        let middlewares = compiler.get_middlewares();
        middlewares.apply_on_module_info(&mut module);

        let compile_info = CompileModuleInfo {
            module: Arc::new(module),
            features,
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

        let data_initializers = translation
            .data_initializers
            .iter()
            .map(OwnedDataInitializer::new)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        // Synthesize a custom section to hold the libcall trampolines.
        let mut function_frame_info = PrimaryMap::with_capacity(compilation.functions.len());
        let mut function_bodies = PrimaryMap::with_capacity(compilation.functions.len());
        let mut function_relocations = PrimaryMap::with_capacity(compilation.functions.len());
        for (_, func) in compilation.functions.into_iter() {
            function_bodies.push(func.body);
            function_relocations.push(func.relocations);
            function_frame_info.push(func.frame_info);
        }
        let mut custom_sections = compilation.custom_sections.clone();
        let mut custom_section_relocations = compilation
            .custom_sections
            .iter()
            .map(|(_, section)| section.relocations.clone())
            .collect::<PrimaryMap<SectionIndex, _>>();
        let libcall_trampolines_section = make_libcall_trampolines(target);
        custom_section_relocations.push(libcall_trampolines_section.relocations.clone());
        let libcall_trampolines = custom_sections.push(libcall_trampolines_section);
        let libcall_trampoline_len = libcall_trampoline_len(target) as u32;
        let cpu_features = compiler.get_cpu_features_used(target.cpu_features());

        let serializable_compilation = SerializableCompilation {
            function_bodies,
            function_relocations,
            function_frame_info,
            function_call_trampolines: compilation.function_call_trampolines,
            dynamic_function_trampolines: compilation.dynamic_function_trampolines,
            custom_sections,
            custom_section_relocations,
            debug: compilation.debug,
            libcall_trampolines,
            libcall_trampoline_len,
        };
        let serializable = SerializableModule {
            compilation: serializable_compilation,
            compile_info,
            data_initializers,
            cpu_features: cpu_features.as_u64(),
        };
        Ok(Self { serializable })
    }

    /// Compile a data buffer into a `ArtifactBuild`, which may then be instantiated.
    #[cfg(not(feature = "compiler"))]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        _inner_engine: &mut EngineInner,
        _data: &[u8],
        _target: &Target,
        _memory_styles: PrimaryMap<MemoryIndex, MemoryStyle>,
        _table_styles: PrimaryMap<TableIndex, TableStyle>,
    ) -> Result<Self, CompileError> {
        Err(CompileError::Codegen(
            "Compilation is not enabled in the engine".to_string(),
        ))
    }

    /// Create a new ArtifactBuild from a SerializableModule
    pub fn from_serializable(serializable: SerializableModule) -> Self {
        Self { serializable }
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
    pub fn get_function_relocations(&self) -> &PrimaryMap<LocalFunctionIndex, Vec<Relocation>> {
        &self.serializable.compilation.function_relocations
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
    pub fn get_debug_ref(&self) -> Option<&Dwarf> {
        self.serializable.compilation.debug.as_ref()
    }

    /// Get Function Relocations ref
    pub fn get_frame_info_ref(&self) -> &PrimaryMap<LocalFunctionIndex, CompiledFunctionFrameInfo> {
        &self.serializable.compilation.function_frame_info
    }
}

impl<'a> ArtifactCreate<'a> for ArtifactBuild {
    type OwnedDataInitializer = &'a OwnedDataInitializer;
    type OwnedDataInitializerIterator = core::slice::Iter<'a, OwnedDataInitializer>;

    fn create_module_info(&self) -> Arc<ModuleInfo> {
        self.serializable.compile_info.module.clone()
    }

    fn set_module_info_name(&mut self, name: String) -> bool {
        Arc::get_mut(&mut self.serializable.compile_info.module).map_or(false, |mut module_info| {
            module_info.name = Some(name.to_string());
            true
        })
    }

    fn module_info(&self) -> &ModuleInfo {
        &self.serializable.compile_info.module
    }

    fn features(&self) -> &Features {
        &self.serializable.compile_info.features
    }

    fn cpu_features(&self) -> EnumSet<CpuFeature> {
        EnumSet::from_u64(self.serializable.cpu_features)
    }

    fn data_initializers(&'a self) -> Self::OwnedDataInitializerIterator {
        self.serializable.data_initializers.iter()
    }

    fn memory_styles(&self) -> &PrimaryMap<MemoryIndex, MemoryStyle> {
        &self.serializable.compile_info.memory_styles
    }

    fn table_styles(&self) -> &PrimaryMap<TableIndex, TableStyle> {
        &self.serializable.compile_info.table_styles
    }

    fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        serialize_module(&self.serializable)
    }
}

/// Module loaded from an archive. Since `CompileModuleInfo` is part of the public
/// interface of this crate and has to be mutable, it has to be deserialized completely.
pub struct ModuleFromArchive<'a> {
    /// The main serializable compilation object
    pub compilation: &'a ArchivedSerializableCompilation,
    /// Datas initializers
    pub data_initializers: &'a rkyv::Archived<Box<[OwnedDataInitializer]>>,
    /// CPU Feature flags for this compilation
    pub cpu_features: u64,

    // Keep the original module around for re-serialization
    original_module: &'a ArchivedSerializableModule,
}

impl<'a> ModuleFromArchive<'a> {
    /// Create a new `ModuleFromArchive` from the archived version of a `SerializableModule`
    pub fn from_serializable_module(
        module: &'a ArchivedSerializableModule,
    ) -> Result<Self, DeserializeError> {
        Ok(Self {
            compilation: &module.compilation,
            data_initializers: &module.data_initializers,
            cpu_features: module.cpu_features,
            original_module: module,
        })
    }
}

self_cell!(
    struct ArtifactBuildFromArchiveCell {
        owner: OwnedBuffer,

        #[covariant]
        dependent: ModuleFromArchive,
    }
);

/// A compiled wasm module that was loaded from a serialized archive.
pub struct ArtifactBuildFromArchive {
    cell: ArtifactBuildFromArchiveCell,

    /// Compilation informations
    compile_info: CompileModuleInfo,
}

impl ArtifactBuildFromArchive {
    pub(crate) fn try_new(
        buffer: OwnedBuffer,
        module_builder: impl FnOnce(
            &OwnedBuffer,
        ) -> Result<&ArchivedSerializableModule, DeserializeError>,
    ) -> Result<Self, DeserializeError> {
        let mut compile_info = MaybeUninit::uninit();

        let cell = ArtifactBuildFromArchiveCell::try_new(buffer, |buffer| {
            let module = module_builder(buffer)?;
            let mut deserializer = SharedDeserializeMap::new();
            compile_info = MaybeUninit::new(
                rkyv::Deserialize::deserialize(&module.compile_info, &mut deserializer)
                    .map_err(|e| DeserializeError::CorruptedBinary(format!("{:?}", e)))?,
            );
            ModuleFromArchive::from_serializable_module(module)
        })?;

        // Safety: we know the lambda will execute before getting here and assign both values
        let compile_info = unsafe { compile_info.assume_init() };
        Ok(Self { cell, compile_info })
    }

    /// Get Functions Bodies ref
    pub fn get_function_bodies_ref(&self) -> &ArchivedPrimaryMap<LocalFunctionIndex, FunctionBody> {
        &self.cell.borrow_dependent().compilation.function_bodies
    }

    /// Get Functions Call Trampolines ref
    pub fn get_function_call_trampolines_ref(
        &self,
    ) -> &ArchivedPrimaryMap<SignatureIndex, FunctionBody> {
        &self
            .cell
            .borrow_dependent()
            .compilation
            .function_call_trampolines
    }

    /// Get Dynamic Functions Call Trampolines ref
    pub fn get_dynamic_function_trampolines_ref(
        &self,
    ) -> &ArchivedPrimaryMap<FunctionIndex, FunctionBody> {
        &self
            .cell
            .borrow_dependent()
            .compilation
            .dynamic_function_trampolines
    }

    /// Get Custom Sections ref
    pub fn get_custom_sections_ref(&self) -> &ArchivedPrimaryMap<SectionIndex, CustomSection> {
        &self.cell.borrow_dependent().compilation.custom_sections
    }

    /// Get Function Relocations
    pub fn get_function_relocations(
        &self,
    ) -> &ArchivedPrimaryMap<LocalFunctionIndex, Vec<Relocation>> {
        &self
            .cell
            .borrow_dependent()
            .compilation
            .function_relocations
    }

    /// Get Function Relocations ref
    pub fn get_custom_section_relocations_ref(
        &self,
    ) -> &ArchivedPrimaryMap<SectionIndex, Vec<Relocation>> {
        &self
            .cell
            .borrow_dependent()
            .compilation
            .custom_section_relocations
    }

    /// Get LibCall Trampoline Section Index
    pub fn get_libcall_trampolines(&self) -> SectionIndex {
        self.cell.borrow_dependent().compilation.libcall_trampolines
    }

    /// Get LibCall Trampoline Length
    pub fn get_libcall_trampoline_len(&self) -> usize {
        self.cell
            .borrow_dependent()
            .compilation
            .libcall_trampoline_len as usize
    }

    /// Get Debug optional Dwarf ref
    pub fn get_debug_ref(&self) -> Option<&Dwarf> {
        match self.cell.borrow_dependent().compilation.debug {
            ArchivedOption::Some(ref x) => Some(x),
            ArchivedOption::None => None,
        }
    }

    /// Get Function Relocations ref
    pub fn deserialize_frame_info_ref(
        &self,
    ) -> Result<PrimaryMap<LocalFunctionIndex, CompiledFunctionFrameInfo>, DeserializeError> {
        let mut deserializer = SharedDeserializeMap::new();
        rkyv::Deserialize::deserialize(
            &self.cell.borrow_dependent().compilation.function_frame_info,
            &mut deserializer,
        )
        .map_err(|e| DeserializeError::CorruptedBinary(format!("{:?}", e)))
    }
}

impl<'a> ArtifactCreate<'a> for ArtifactBuildFromArchive {
    type OwnedDataInitializer = &'a ArchivedOwnedDataInitializer;
    type OwnedDataInitializerIterator = core::slice::Iter<'a, ArchivedOwnedDataInitializer>;

    fn create_module_info(&self) -> Arc<ModuleInfo> {
        self.compile_info.module.clone()
    }

    fn set_module_info_name(&mut self, name: String) -> bool {
        Arc::get_mut(&mut self.compile_info.module).map_or(false, |mut module_info| {
            module_info.name = Some(name.to_string());
            true
        })
    }

    fn module_info(&self) -> &ModuleInfo {
        &self.compile_info.module
    }

    fn features(&self) -> &Features {
        &self.compile_info.features
    }

    fn cpu_features(&self) -> EnumSet<CpuFeature> {
        EnumSet::from_u64(self.cell.borrow_dependent().cpu_features)
    }

    fn data_initializers(&'a self) -> Self::OwnedDataInitializerIterator {
        self.cell.borrow_dependent().data_initializers.iter()
    }

    fn memory_styles(&self) -> &PrimaryMap<MemoryIndex, MemoryStyle> {
        &self.compile_info.memory_styles
    }

    fn table_styles(&self) -> &PrimaryMap<TableIndex, TableStyle> {
        &self.compile_info.table_styles
    }

    fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        // We could have stored the original bytes, but since the module info name
        // is mutable, we have to assume the data may have changed and serialize
        // everything all over again. Also, to be able to serialize, first we have
        // to deserialize completely. Luckily, serializing a module that was already
        // deserialized from a file makes little sense, so hopefully, this is not a
        // common use-case.

        let mut deserializer = SharedDeserializeMap::new();
        let mut module: SerializableModule = rkyv::Deserialize::deserialize(
            self.cell.borrow_dependent().original_module,
            &mut deserializer,
        )
        .map_err(|e| SerializeError::Generic(e.to_string()))?;
        module.compile_info = self.compile_info.clone();
        serialize_module(&module)
    }
}

fn serialize_module(module: &SerializableModule) -> Result<Vec<u8>, SerializeError> {
    let serialized_data = module.serialize()?;
    assert!(std::mem::align_of::<SerializableModule>() <= MetadataHeader::ALIGN);

    let mut metadata_binary = vec![];
    metadata_binary.extend(ArtifactBuild::MAGIC_HEADER);
    metadata_binary.extend(MetadataHeader::new(serialized_data.len()).into_bytes());
    metadata_binary.extend(serialized_data);
    Ok(metadata_binary)
}
