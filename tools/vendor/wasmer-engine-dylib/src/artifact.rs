//! Define `DylibArtifact` to allow compiling and instantiating
//! to be done as separate steps.

use crate::engine::{DylibEngine, DylibEngineInner};
use crate::serialize::ModuleMetadata;
use crate::trampoline::{emit_trampolines, fill_trampoline_table, WASMER_TRAMPOLINES_SYMBOL};
use enumset::EnumSet;
use libloading::{Library, Symbol as LibrarySymbol};
use loupe::MemoryUsage;
use object::{write::CoffExportStyle, BinaryFormat};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
#[cfg(feature = "compiler")]
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use tracing::log::error;
#[cfg(feature = "compiler")]
use tracing::trace;
use wasmer_artifact::ArtifactCreate;
use wasmer_compiler::{
    Architecture, CompileError, CompiledFunctionFrameInfo, CpuFeature, Features,
    FunctionAddressMap, OperatingSystem, Symbol, SymbolRegistry, Triple,
};
#[cfg(feature = "compiler")]
use wasmer_compiler::{
    CompileModuleInfo, Compiler, FunctionBodyData, ModuleEnvironment, ModuleMiddlewareChain,
    ModuleTranslationState,
};
use wasmer_engine::{
    register_frame_info, Artifact, DeserializeError, FunctionExtent, GlobalFrameInfoRegistration,
    InstantiationError, MetadataHeader, SerializeError,
};
#[cfg(feature = "compiler")]
use wasmer_engine::{Engine, Tunables};
#[cfg(feature = "compiler")]
use wasmer_object::{emit_compilation, emit_data, get_object_for_target};
use wasmer_types::entity::{BoxedSlice, PrimaryMap};
#[cfg(feature = "compiler")]
use wasmer_types::DataInitializer;
use wasmer_types::{
    FunctionIndex, LocalFunctionIndex, MemoryIndex, ModuleInfo, OwnedDataInitializer,
    SignatureIndex, TableIndex,
};
use wasmer_vm::{
    FuncDataRegistry, FunctionBodyPtr, MemoryStyle, TableStyle, VMFunctionBody,
    VMSharedSignatureIndex, VMTrampoline,
};

/// A compiled Wasm module, ready to be instantiated.
#[derive(MemoryUsage)]
pub struct DylibArtifact {
    dylib_path: PathBuf,
    is_temporary: bool,
    metadata: ModuleMetadata,
    finished_functions: BoxedSlice<LocalFunctionIndex, FunctionBodyPtr>,
    #[loupe(skip)]
    finished_function_call_trampolines: BoxedSlice<SignatureIndex, VMTrampoline>,
    finished_dynamic_function_trampolines: BoxedSlice<FunctionIndex, FunctionBodyPtr>,
    func_data_registry: Arc<FuncDataRegistry>,
    signatures: BoxedSlice<SignatureIndex, VMSharedSignatureIndex>,
    frame_info_registration: Mutex<Option<GlobalFrameInfoRegistration>>,
}

impl Drop for DylibArtifact {
    fn drop(&mut self) {
        if self.is_temporary {
            if let Err(err) = std::fs::remove_file(&self.dylib_path) {
                error!("cannot delete the temporary dylib artifact: {}", err);
            }
        }
    }
}

fn to_compile_error(err: impl Error) -> CompileError {
    CompileError::Codegen(err.to_string())
}

const WASMER_METADATA_SYMBOL: &[u8] = b"WASMER_METADATA";

impl DylibArtifact {
    // Mach-O header in iOS/Mac
    #[allow(dead_code)]
    const MAGIC_HEADER_MH_CIGAM_64: &'static [u8] = &[207, 250, 237, 254];

    // ELF Magic header for Linux (32 bit)
    #[allow(dead_code)]
    const MAGIC_HEADER_ELF_32: &'static [u8] = &[0x7f, b'E', b'L', b'F', 1];

    // ELF Magic header for Linux (64 bit)
    #[allow(dead_code)]
    const MAGIC_HEADER_ELF_64: &'static [u8] = &[0x7f, b'E', b'L', b'F', 2];

    // COFF Magic header for Windows (64 bit)
    #[allow(dead_code)]
    const MAGIC_HEADER_COFF_64: &'static [u8] = &[b'M', b'Z'];

    /// Check if the provided bytes look like `DylibArtifact`.
    ///
    /// This means, if the bytes look like a shared object file in the target
    /// system.
    pub fn is_deserializable(bytes: &[u8]) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(all(target_pointer_width = "64", target_vendor="apple"))] {
                bytes.starts_with(Self::MAGIC_HEADER_MH_CIGAM_64)
            }
            else if #[cfg(all(target_pointer_width = "64", target_os="linux"))] {
                bytes.starts_with(Self::MAGIC_HEADER_ELF_64)
            }
            else if #[cfg(all(target_pointer_width = "32", target_os="linux"))] {
                bytes.starts_with(Self::MAGIC_HEADER_ELF_32)
            }
            else if #[cfg(all(target_pointer_width = "64", target_os="windows"))] {
                bytes.starts_with(Self::MAGIC_HEADER_COFF_64)
            }
            else {
                false
            }
        }
    }

    #[cfg(feature = "compiler")]
    /// Generate a compilation
    fn generate_metadata<'data>(
        data: &'data [u8],
        features: &Features,
        compiler: &dyn Compiler,
        tunables: &dyn Tunables,
    ) -> Result<
        (
            CompileModuleInfo,
            PrimaryMap<LocalFunctionIndex, FunctionBodyData<'data>>,
            Vec<DataInitializer<'data>>,
            Option<ModuleTranslationState>,
        ),
        CompileError,
    > {
        let environ = ModuleEnvironment::new();
        let translation = environ.translate(data).map_err(CompileError::Wasm)?;

        // We try to apply the middleware first
        let mut module = translation.module;
        let middlewares = compiler.get_middlewares();
        middlewares.apply_on_module_info(&mut module);

        let memory_styles: PrimaryMap<MemoryIndex, MemoryStyle> = module
            .memories
            .values()
            .map(|memory_type| tunables.memory_style(memory_type))
            .collect();
        let table_styles: PrimaryMap<TableIndex, TableStyle> = module
            .tables
            .values()
            .map(|table_type| tunables.table_style(table_type))
            .collect();

        let compile_info = CompileModuleInfo {
            module: Arc::new(module),
            features: features.clone(),
            memory_styles,
            table_styles,
        };
        Ok((
            compile_info,
            translation.function_body_inputs,
            translation.data_initializers,
            translation.module_translation_state,
        ))
    }

    /// Compile a data buffer into a `DylibArtifact`, which may
    /// then be instantiated.
    #[cfg(feature = "compiler")]
    pub fn new(
        engine: &DylibEngine,
        data: &[u8],
        tunables: &dyn Tunables,
    ) -> Result<Self, CompileError> {
        let mut engine_inner = engine.inner_mut();
        let target = engine.target();
        let compiler = engine_inner.compiler()?;
        let (compile_info, function_body_inputs, data_initializers, module_translation) =
            Self::generate_metadata(data, engine_inner.features(), compiler, tunables)?;

        let data_initializers = data_initializers
            .iter()
            .map(OwnedDataInitializer::new)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let target_triple = target.triple();

        /*
        // We construct the function body lengths
        let function_body_lengths = compilation
            .get_function_bodies()
            .values()
            .map(|function_body| function_body.body.len() as u64)
            .map(|_function_body| 0u64)
            .collect::<PrimaryMap<LocalFunctionIndex, u64>>();
         */

        // TODO: we currently supply all-zero function body lengths.
        // We don't know the lengths until they're compiled, yet we have to
        // supply the metadata as an input to the compile.
        let function_body_lengths = function_body_inputs
            .keys()
            .map(|_function_body| 0u64)
            .collect::<PrimaryMap<LocalFunctionIndex, u64>>();

        let function_frame_info = None;

        let mut metadata = ModuleMetadata {
            compile_info,
            function_frame_info,
            prefix: engine_inner.get_prefix(data),
            data_initializers,
            function_body_lengths,
            cpu_features: target.cpu_features().as_u64(),
        };

        let serialized_data = metadata.serialize()?;

        let mut metadata_binary = vec![];
        metadata_binary.extend(MetadataHeader::new(serialized_data.len()));
        metadata_binary.extend(serialized_data);

        let (compile_info, symbol_registry) = metadata.split();

        let maybe_obj_bytes = compiler.experimental_native_compile_module(
            target,
            compile_info,
            module_translation.as_ref().unwrap(),
            &function_body_inputs,
            &symbol_registry,
            &metadata_binary,
        );

        let mut extra_filepath = None;
        let filepath = match maybe_obj_bytes {
            Some(obj_bytes) => {
                extra_filepath = {
                    // Create a separate object file with the trampolines.
                    let mut obj = get_object_for_target(target_triple).map_err(to_compile_error)?;
                    emit_trampolines(&mut obj, engine.target());
                    if obj.format() == BinaryFormat::Coff {
                        obj.add_coff_exports(CoffExportStyle::Gnu);
                    }
                    let file = tempfile::Builder::new()
                        .prefix("wasmer_dylib_")
                        .suffix(".o")
                        .tempfile()
                        .map_err(to_compile_error)?;

                    // Re-open it.
                    let (mut file, filepath) = file.keep().map_err(to_compile_error)?;
                    let obj_bytes = obj.write().map_err(to_compile_error)?;
                    file.write_all(&obj_bytes).map_err(to_compile_error)?;
                    Some(filepath)
                };

                // Write the object file generated by the compiler.
                let obj_bytes = obj_bytes?;
                let file = tempfile::Builder::new()
                    .prefix("wasmer_dylib_")
                    .suffix(".o")
                    .tempfile()
                    .map_err(to_compile_error)?;

                // Re-open it.
                let (mut file, filepath) = file.keep().map_err(to_compile_error)?;
                file.write_all(&obj_bytes).map_err(to_compile_error)?;
                filepath
            }
            None => {
                let compilation = compiler.compile_module(
                    target,
                    compile_info,
                    module_translation.as_ref().unwrap(),
                    function_body_inputs,
                )?;
                let mut obj = get_object_for_target(target_triple).map_err(to_compile_error)?;
                emit_trampolines(&mut obj, engine.target());
                emit_data(
                    &mut obj,
                    WASMER_METADATA_SYMBOL,
                    &metadata_binary,
                    MetadataHeader::ALIGN as u64,
                )
                .map_err(to_compile_error)?;

                let frame_info = compilation.get_frame_info();

                emit_compilation(&mut obj, compilation, &symbol_registry, target_triple)
                    .map_err(to_compile_error)?;
                if obj.format() == BinaryFormat::Coff {
                    obj.add_coff_exports(CoffExportStyle::Gnu);
                }
                let file = tempfile::Builder::new()
                    .prefix("wasmer_dylib_")
                    .suffix(".o")
                    .tempfile()
                    .map_err(to_compile_error)?;

                metadata.function_frame_info = Some(frame_info);

                // Re-open it.
                let (mut file, filepath) = file.keep().map_err(to_compile_error)?;
                let obj_bytes = obj.write().map_err(to_compile_error)?;

                file.write_all(&obj_bytes).map_err(to_compile_error)?;
                filepath
            }
        };

        let output_filepath = {
            let suffix = format!(".{}", Self::get_default_extension(target_triple));
            let shared_file = tempfile::Builder::new()
                .prefix("wasmer_dylib_")
                .suffix(&suffix)
                .tempfile()
                .map_err(to_compile_error)?;
            shared_file
                .into_temp_path()
                .keep()
                .map_err(to_compile_error)?
        };

        let is_cross_compiling = engine_inner.is_cross_compiling();
        let target_triple_str = {
            let into_str = target_triple.to_string();
            // We have to adapt the target triple string, because otherwise
            // Apple's clang will not recognize it.
            if into_str == "aarch64-apple-darwin" {
                "arm64-apple-darwin".to_string()
            } else {
                into_str
            }
        };

        // Set 'isysroot' clang flag if compiling to iOS target
        let ios_compile_target = target_triple.operating_system == OperatingSystem::Ios;
        let ios_sdk_flag = {
            if ios_compile_target {
                if target_triple.architecture == Architecture::X86_64 {
                    "-isysroot/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator.sdk"
                } else {
                    "-isysroot/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk"
                }
            } else {
                ""
            }
        };
        let ios_sdk_lib = {
            if ios_compile_target {
                "-lSystem"
            } else {
                ""
            }
        };

        // Get the location of the 'ld' linker for clang
        let fuse_linker = {
            let ld_install = which::which("ld");
            if ios_compile_target && ld_install.is_ok() {
                ld_install.unwrap().into_os_string().into_string().unwrap()
            } else {
                "lld".to_string()
            }
        };

        let cross_compiling_args: Vec<String> = if is_cross_compiling {
            vec![
                format!("--target={}", target_triple_str),
                format!("-fuse-ld={}", fuse_linker),
                "-nodefaultlibs".to_string(),
                "-nostdlib".to_string(),
                ios_sdk_flag.to_string(),
                ios_sdk_lib.to_string(),
            ]
        } else {
            // We are explicit on the target when the host system is
            // Apple Silicon, otherwise compilation fails.
            if target_triple_str == "arm64-apple-darwin" {
                vec![format!("--target={}", target_triple_str)]
            } else {
                vec![]
            }
        };
        let target_args = match (target_triple.operating_system, is_cross_compiling) {
            (OperatingSystem::Windows, true) => vec!["-Wl,/force:unresolved,/noentry"],
            (OperatingSystem::Windows, false) => vec!["-Wl,-undefined,dynamic_lookup"],
            _ => vec!["-nostartfiles", "-Wl,-undefined,dynamic_lookup"],
        };
        trace!(
            "Compiling for target {} from host {}",
            target_triple_str,
            Triple::host().to_string(),
        );

        let linker = engine_inner.linker().executable();
        let output = Command::new(linker)
            .arg(&filepath)
            .args(&extra_filepath)
            .arg("-o")
            .arg(&output_filepath)
            .args(&target_args)
            // .args(&wasmer_symbols)
            .arg("-shared")
            .args(&cross_compiling_args)
            .arg("-v")
            .output()
            .map_err(to_compile_error);

        if fs::metadata(&filepath).is_ok() {
            fs::remove_file(filepath).map_err(to_compile_error)?;
        }
        if let Some(filepath) = extra_filepath {
            if fs::metadata(&filepath).is_ok() {
                fs::remove_file(filepath).map_err(to_compile_error)?;
            }
        }

        let output = output?;

        if !output.status.success() {
            return Err(CompileError::Codegen(format!(
                "Shared object file generator failed with:\nstderr:{}\nstdout:{}",
                String::from_utf8_lossy(&output.stderr).trim_end(),
                String::from_utf8_lossy(&output.stdout).trim_end()
            )));
        }

        trace!("gcc command result {:?}", output);

        let mut artifact = if is_cross_compiling {
            Self::from_parts_crosscompiled(metadata, output_filepath)
        } else {
            let lib = unsafe { Library::new(&output_filepath).map_err(to_compile_error)? };
            Self::from_parts(&mut engine_inner, metadata, output_filepath, lib)
        }?;
        artifact.is_temporary = true;

        Ok(artifact)
    }

    /// Get the default extension when serializing this artifact
    pub fn get_default_extension(triple: &Triple) -> &'static str {
        match triple.operating_system {
            OperatingSystem::Windows => "dll",
            OperatingSystem::Darwin | OperatingSystem::Ios | OperatingSystem::MacOSX { .. } => {
                "dylib"
            }
            _ => "so",
        }
    }

    /// Construct a `DylibArtifact` from component parts.
    pub fn from_parts_crosscompiled(
        metadata: ModuleMetadata,
        dylib_path: PathBuf,
    ) -> Result<Self, CompileError> {
        let finished_functions: PrimaryMap<LocalFunctionIndex, FunctionBodyPtr> = PrimaryMap::new();
        let finished_function_call_trampolines: PrimaryMap<SignatureIndex, VMTrampoline> =
            PrimaryMap::new();
        let finished_dynamic_function_trampolines: PrimaryMap<FunctionIndex, FunctionBodyPtr> =
            PrimaryMap::new();
        let signatures: PrimaryMap<SignatureIndex, VMSharedSignatureIndex> = PrimaryMap::new();
        Ok(Self {
            dylib_path,
            is_temporary: false,
            metadata,
            finished_functions: finished_functions.into_boxed_slice(),
            finished_function_call_trampolines: finished_function_call_trampolines
                .into_boxed_slice(),
            finished_dynamic_function_trampolines: finished_dynamic_function_trampolines
                .into_boxed_slice(),
            func_data_registry: Arc::new(FuncDataRegistry::new()),
            signatures: signatures.into_boxed_slice(),
            frame_info_registration: Mutex::new(None),
        })
    }

    /// Construct a `DylibArtifact` from component parts.
    pub fn from_parts(
        engine_inner: &mut DylibEngineInner,
        metadata: ModuleMetadata,
        dylib_path: PathBuf,
        lib: Library,
    ) -> Result<Self, CompileError> {
        unsafe {
            let trampolines_symbol: LibrarySymbol<usize> = lib
                .get(WASMER_TRAMPOLINES_SYMBOL)
                .expect("missing WASMER_TRAMPOLINES symbol");
            fill_trampoline_table(trampolines_symbol.into_raw().into_raw() as *mut usize);
        }

        let mut finished_functions: PrimaryMap<LocalFunctionIndex, FunctionBodyPtr> =
            PrimaryMap::new();
        for (function_local_index, _function_len) in metadata.function_body_lengths.iter() {
            let function_name = metadata
                .get_symbol_registry()
                .symbol_to_name(Symbol::LocalFunction(function_local_index));
            unsafe {
                // We use a fake function signature `fn()` because we just
                // want to get the function address.
                let func: LibrarySymbol<unsafe extern "C" fn()> = lib
                    .get(function_name.as_bytes())
                    .map_err(to_compile_error)?;
                finished_functions.push(FunctionBodyPtr(
                    func.into_raw().into_raw() as *const VMFunctionBody
                ));
            }
        }

        // Retrieve function call trampolines
        let mut finished_function_call_trampolines: PrimaryMap<SignatureIndex, VMTrampoline> =
            PrimaryMap::with_capacity(metadata.compile_info.module.signatures.len());
        for sig_index in metadata.compile_info.module.signatures.keys() {
            let function_name = metadata
                .get_symbol_registry()
                .symbol_to_name(Symbol::FunctionCallTrampoline(sig_index));
            unsafe {
                let trampoline: LibrarySymbol<VMTrampoline> = lib
                    .get(function_name.as_bytes())
                    .map_err(to_compile_error)?;
                let raw = *trampoline.into_raw();
                finished_function_call_trampolines.push(raw);
            }
        }

        // Retrieve dynamic function trampolines (only for imported functions)
        let mut finished_dynamic_function_trampolines: PrimaryMap<FunctionIndex, FunctionBodyPtr> =
            PrimaryMap::with_capacity(metadata.compile_info.module.num_imported_functions);
        for func_index in metadata
            .compile_info
            .module
            .functions
            .keys()
            .take(metadata.compile_info.module.num_imported_functions)
        {
            let function_name = metadata
                .get_symbol_registry()
                .symbol_to_name(Symbol::DynamicFunctionTrampoline(func_index));
            unsafe {
                let trampoline: LibrarySymbol<unsafe extern "C" fn()> = lib
                    .get(function_name.as_bytes())
                    .map_err(to_compile_error)?;
                finished_dynamic_function_trampolines.push(FunctionBodyPtr(
                    trampoline.into_raw().into_raw() as *const VMFunctionBody,
                ));
            }
        }

        // Compute indices into the shared signature table.
        let signatures = {
            metadata
                .compile_info
                .module
                .signatures
                .values()
                .map(|sig| engine_inner.signatures().register(sig))
                .collect::<PrimaryMap<_, _>>()
        };

        engine_inner.add_library(lib);

        Ok(Self {
            dylib_path,
            is_temporary: false,
            metadata,
            finished_functions: finished_functions.into_boxed_slice(),
            finished_function_call_trampolines: finished_function_call_trampolines
                .into_boxed_slice(),
            finished_dynamic_function_trampolines: finished_dynamic_function_trampolines
                .into_boxed_slice(),
            func_data_registry: engine_inner.func_data().clone(),
            signatures: signatures.into_boxed_slice(),
            frame_info_registration: Mutex::new(None),
        })
    }

    /// Compile a data buffer into a `DylibArtifact`, which may
    /// then be instantiated.
    #[cfg(not(feature = "compiler"))]
    pub fn new(_engine: &DylibEngine, _data: &[u8]) -> Result<Self, CompileError> {
        Err(CompileError::Codegen(
            "Compilation is not enabled in the engine".to_string(),
        ))
    }

    /// Deserialize a `DylibArtifact` from bytes.
    ///
    /// # Safety
    ///
    /// The bytes must represent a serialized WebAssembly module.
    pub unsafe fn deserialize(
        engine: &DylibEngine,
        bytes: &[u8],
    ) -> Result<Self, DeserializeError> {
        if !Self::is_deserializable(bytes) {
            return Err(DeserializeError::Incompatible(
                "The provided bytes are not in any native format Wasmer can understand".to_string(),
            ));
        }
        // Dump the bytes into a file, so we can read it with our `dlopen`
        let named_file = NamedTempFile::new()?;
        let (mut file, path) = named_file.keep().map_err(|e| e.error)?;
        file.write_all(bytes)?;
        // We already checked for the header, so we don't need
        // to check again.
        let mut artifact = Self::deserialize_from_file_unchecked(engine, &path)?;
        artifact.is_temporary = true;

        Ok(artifact)
    }

    /// Deserialize a `DylibArtifact` from a file path.
    ///
    /// # Safety
    ///
    /// The file's content must represent a serialized WebAssembly module.
    pub unsafe fn deserialize_from_file(
        engine: &DylibEngine,
        path: &Path,
    ) -> Result<Self, DeserializeError> {
        let mut file = File::open(&path)?;
        let mut buffer = [0; 5];
        // read up to 5 bytes
        file.read_exact(&mut buffer)?;
        if !Self::is_deserializable(&buffer) {
            return Err(DeserializeError::Incompatible(
                "The provided bytes are not in any native format Wasmer can understand".to_string(),
            ));
        }
        Self::deserialize_from_file_unchecked(engine, path)
    }

    /// Deserialize a `DylibArtifact` from a file path (unchecked).
    ///
    /// # Safety
    ///
    /// The file's content must represent a serialized WebAssembly module.
    pub unsafe fn deserialize_from_file_unchecked(
        engine: &DylibEngine,
        path: &Path,
    ) -> Result<Self, DeserializeError> {
        let lib = Library::new(&path).map_err(|e| {
            DeserializeError::CorruptedBinary(format!("Library loading failed: {}", e))
        })?;
        let shared_path: PathBuf = PathBuf::from(path);
        let metadata_symbol: LibrarySymbol<*mut [u8; MetadataHeader::LEN]> =
            lib.get(WASMER_METADATA_SYMBOL).map_err(|e| {
                DeserializeError::CorruptedBinary(format!(
                    "The provided object file doesn't seem to be generated by Wasmer: {}",
                    e
                ))
            })?;
        use std::slice;

        let metadata = &**metadata_symbol;
        let metadata_len = MetadataHeader::parse(metadata)?;
        let metadata_slice: &'static [u8] =
            slice::from_raw_parts(metadata.as_ptr().add(MetadataHeader::LEN), metadata_len);

        let metadata = ModuleMetadata::deserialize(metadata_slice)?;

        let mut engine_inner = engine.inner_mut();

        Self::from_parts(&mut engine_inner, metadata, shared_path, lib)
            .map_err(DeserializeError::Compiler)
    }

    /// Used in test deserialize metadata is correct
    pub fn metadata(&self) -> &ModuleMetadata {
        &self.metadata
    }
}

impl ArtifactCreate for DylibArtifact {
    fn module(&self) -> Arc<ModuleInfo> {
        self.metadata.compile_info.module.clone()
    }

    fn module_ref(&self) -> &ModuleInfo {
        &self.metadata.compile_info.module
    }

    fn module_mut(&mut self) -> Option<&mut ModuleInfo> {
        Arc::get_mut(&mut self.metadata.compile_info.module)
    }

    fn features(&self) -> &Features {
        &self.metadata.compile_info.features
    }

    fn cpu_features(&self) -> enumset::EnumSet<CpuFeature> {
        EnumSet::from_u64(self.metadata.cpu_features)
    }

    fn data_initializers(&self) -> &[OwnedDataInitializer] {
        &*self.metadata.data_initializers
    }

    fn memory_styles(&self) -> &PrimaryMap<MemoryIndex, MemoryStyle> {
        &self.metadata.compile_info.memory_styles
    }

    fn table_styles(&self) -> &PrimaryMap<TableIndex, TableStyle> {
        &self.metadata.compile_info.table_styles
    }

    /// Serialize a `DylibArtifact`.
    fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(std::fs::read(&self.dylib_path)?)
    }

    /// Serialize a `DylibArtifact` to a portable file
    #[cfg(feature = "compiler")]
    fn serialize_to_file(&self, path: &Path) -> Result<(), SerializeError> {
        let serialized = self.serialize()?;
        std::fs::write(&path, serialized)?;

        /*
        When you write the artifact to a new file it still has the 'Mach-O Identifier'
        of the original file, and so this can causes linker issues when adding
        the new file to an XCode project.

        The below code renames the ID of the file so that it references itself through
        an @executable_path prefix. Basically it tells XCode to find this file
        inside of the projects' list of 'linked executables'.

        You need to be running MacOS for the following to actually work though.
        */
        let has_extension = path.extension().is_some();
        if has_extension && path.extension().unwrap() == "dylib" {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let parent_dir = path.parent().unwrap();
            let absolute_path = std::fs::canonicalize(&parent_dir)
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap();

            Command::new("install_name_tool")
                .arg("-id")
                .arg(format!("@executable_path/{}", &filename))
                .arg(&filename)
                .current_dir(&absolute_path)
                .output()?;
        }

        Ok(())
    }
}
impl Artifact for DylibArtifact {
    fn register_frame_info(&self) {
        let mut info = self.frame_info_registration.lock().unwrap();

        if info.is_some() {
            return;
        }

        // We (reverse) order all the functions by their pointer location.
        // [f9, f6, f7, f8...] and calculate their potential function body size by
        // getting the diff in pointers between functions (since they are all located
        // in the same __text section).

        let min_call_trampolines_pointer = self
            .finished_function_call_trampolines
            .values()
            .map(|t| *t as usize)
            .min()
            .unwrap_or(0);
        let min_dynamic_trampolines_pointer = self
            .finished_dynamic_function_trampolines
            .values()
            .map(|t| **t as usize)
            .min()
            .unwrap_or(0);

        let fp = self.finished_functions.clone();
        let mut function_pointers = fp.into_iter().collect::<Vec<_>>();

        // Sort the keys by the funciton pointer values in reverse order.
        // This way we can get the maximum function lengths (since functions can't overlap in memory)
        function_pointers.sort_by(|(_k1, v1), (_k2, v2)| v2.cmp(v1));
        let mut iter = function_pointers.into_iter();
        let mut function_pointers = iter
            .next()
            .map(|(index, function_pointer)| {
                let fp = **function_pointer as usize;
                // In case we are in the first function pointer (the one with the highest pointer)
                // we try to determine it's bounds (function size) by using the other function trampoline
                // locations.
                let current_size_by_ptr = if fp < min_call_trampolines_pointer {
                    if min_call_trampolines_pointer < min_dynamic_trampolines_pointer
                        || min_dynamic_trampolines_pointer == 0
                    {
                        min_call_trampolines_pointer - fp
                    } else {
                        min_dynamic_trampolines_pointer - fp
                    }
                } else if fp < min_dynamic_trampolines_pointer {
                    min_dynamic_trampolines_pointer - fp
                } else {
                    // The trampoline pointers are before the function.
                    // We can safely assume the function will be at least 16 bits of length.
                    // This is a very hacky assumption, but it makes collisions work perfectly
                    // Since DLOpen simlinks will always be > 16 bits of difference between
                    // two different libraries for the same symbol.
                    // Note: the minmum Mach-O file is 0x1000 bytes and the minimum ELF is 0x0060 bytes
                    16
                };
                let mut prev_pointer = fp;
                // We choose the minimum between the function size given the pointer diff
                // and the emitted size by the address map
                let ptr = function_pointer;
                let length = current_size_by_ptr;
                let first = (
                    index,
                    FunctionExtent {
                        ptr: *ptr,
                        length: length,
                    },
                );
                std::iter::once(first)
                    .chain(iter.map(|(index, function_pointer)| {
                        let fp = **function_pointer as usize;
                        // This assumes we never lay any functions bodies across the usize::MAX..nullptr
                        // wrapping point.
                        // Which is generally true on most OSes, but certainly doesn't have to be true.
                        //
                        // Further reading: https://lwn.net/Articles/342330/ \
                        // "There is one little problem with that reasoning, though: NULL (zero) can
                        // actually be a valid pointer address."
                        let current_size_by_ptr = prev_pointer - fp;

                        prev_pointer = fp;
                        // We choose the minimum between the function size given the pointer diff
                        // and the emitted size by the address map
                        let ptr = function_pointer;
                        let length = current_size_by_ptr;
                        (
                            index,
                            FunctionExtent {
                                ptr: *ptr,
                                length: length,
                            },
                        )
                    }))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // We sort them again, by key this time
        function_pointers.sort_by(|(k1, _v1), (k2, _v2)| k1.cmp(k2));

        let frame_infos = if self.metadata().function_frame_info.is_some() {
            self.metadata().function_frame_info.clone().unwrap()
        } else {
            function_pointers
                .iter()
                .map(|(_, extent)| CompiledFunctionFrameInfo {
                    traps: vec![],
                    address_map: FunctionAddressMap {
                        body_len: extent.length,
                        ..Default::default()
                    },
                })
                .collect::<PrimaryMap<LocalFunctionIndex, _>>()
        };

        let finished_function_extents = function_pointers
            .into_iter()
            .map(|(_, function_extent)| function_extent)
            .collect::<PrimaryMap<LocalFunctionIndex, _>>()
            .into_boxed_slice();

        *info = register_frame_info(
            self.metadata.compile_info.module.clone(),
            &finished_function_extents,
            frame_infos,
        );
    }

    fn finished_functions(&self) -> &BoxedSlice<LocalFunctionIndex, FunctionBodyPtr> {
        &self.finished_functions
    }

    fn finished_function_call_trampolines(&self) -> &BoxedSlice<SignatureIndex, VMTrampoline> {
        &self.finished_function_call_trampolines
    }

    fn finished_dynamic_function_trampolines(&self) -> &BoxedSlice<FunctionIndex, FunctionBodyPtr> {
        &self.finished_dynamic_function_trampolines
    }

    fn signatures(&self) -> &BoxedSlice<SignatureIndex, VMSharedSignatureIndex> {
        &self.signatures
    }

    fn func_data_registry(&self) -> &FuncDataRegistry {
        &self.func_data_registry
    }

    fn preinstantiate(&self) -> Result<(), InstantiationError> {
        Ok(())
    }
}
