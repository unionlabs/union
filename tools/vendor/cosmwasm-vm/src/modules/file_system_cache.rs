use std::fs;
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

use wasmer::{AsEngineRef, DeserializeError, Module, Target};

use crate::checksum::Checksum;
use crate::errors::{VmError, VmResult};

use crate::filesystem::mkdir_p;
use crate::modules::current_wasmer_module_version;

/// Bump this version whenever the module system changes in a way
/// that old stored modules would be corrupt when loaded in the new system.
/// This needs to be done e.g. when switching between the jit/native engine.
///
/// The string is used as a folder and should be named in a way that is
/// easy to interprete for system admins. It should allow easy clearing
/// of old versions.
///
/// See https://github.com/wasmerio/wasmer/issues/2781 for more information
/// on Wasmer's module stability concept.
///
/// ## Version history:
/// - **v1**:<br>
///   cosmwasm_vm < 1.0.0-beta5. This is working well up to Wasmer 2.0.0 as
///   [in wasmvm 1.0.0-beta2](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-beta2/libwasmvm/Cargo.lock#L1412-L1413)
///   and [wasmvm 0.16.3](https://github.com/CosmWasm/wasmvm/blob/v0.16.3/libwasmvm/Cargo.lock#L1408-L1409).
///   Versions that ship with Wasmer 2.1.x such [as wasmvm 1.0.0-beta3](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-beta3/libwasmvm/Cargo.lock#L1534-L1535)
///   to [wasmvm 1.0.0-beta5](https://github.com/CosmWasm/wasmvm/blob/v1.0.0-beta5/libwasmvm/Cargo.lock#L1530-L1531)
///   are broken, i.e. they will crash when reading older v1 modules.
/// - **v2**:<br>
///   Version for cosmwasm_vm 1.0.0-beta5 / wasmvm 1.0.0-beta6 that ships with Wasmer 2.1.1.
/// - **v3**:<br>
///   Version for Wasmer 2.2.0 which contains a [module breaking change to 2.1.x](https://github.com/wasmerio/wasmer/pull/2747).
/// - **v4**:<br>
///   Version for Wasmer 2.3.0 which contains a module breaking change to 2.2.0 that was not reflected in
///   the module header version (<https://github.com/wasmerio/wasmer/issues/3193>). In cosmwasm-vm 1.1.0-1.1.1
///   the old value "v3" is still used along with Wasmer 2.3.0 (bug). From cosmwasm 1.1.2 onwards, this is
///   fixed by bumping to "v4".
/// - **v5**:<br>
///   A change in memory layout of some types in Rust [std] caused
///   [issues with module deserialization](https://github.com/CosmWasm/wasmvm/issues/426).
///   To work around this, the version was bumped to "v5" here to invalidate these corrupt caches.
/// - **v6**:<br>
///   Version for cosmwasm_vm 1.3+ which adds a sub-folder with the target identier for the modules.
/// - **v7**:<br>
///   New version because of Wasmer 2.3.0 -> 4 upgrade.
///   This internally changes how rkyv is used for module serialization, making compatibility unlikely.
/// - **v8**:<br>
///   New version because of Wasmer 4.1.2 -> 4.2.2 upgrade.
///   Module compatibility between Wasmer versions is not guaranteed.
const MODULE_SERIALIZATION_VERSION: &str = "v8";

/// Representation of a directory that contains compiled Wasm artifacts.
pub struct FileSystemCache {
    modules_path: PathBuf,
    /// If true, the cache uses the `*_unchecked` wasmer functions for loading modules from disk.
    unchecked_modules: bool,
}

/// An error type that hides system specific error information
/// to ensure deterministic errors across operating systems.
#[derive(Error, Debug)]
pub enum NewFileSystemCacheError {
    #[error("Could not get metadata of cache path")]
    CouldntGetMetadata,
    #[error("The supplied path is readonly")]
    ReadonlyPath,
    #[error("The supplied path already exists but is no directory")]
    ExistsButNoDirectory,
    #[error("Could not create cache path")]
    CouldntCreatePath,
}

impl FileSystemCache {
    /// Construct a new `FileSystemCache` around the specified directory.
    /// The contents of the cache are stored in sub-versioned directories.
    /// If `unchecked_modules` is set to true, it uses the `*_unchecked`
    /// wasmer functions for loading modules from disk (no validity checks).
    ///
    /// # Safety
    ///
    /// This method is unsafe because there's no way to ensure the artifacts
    /// stored in this cache haven't been corrupted or tampered with.
    pub unsafe fn new(
        base_path: impl Into<PathBuf>,
        unchecked_modules: bool,
    ) -> Result<Self, NewFileSystemCacheError> {
        let base_path: PathBuf = base_path.into();
        if base_path.exists() {
            let metadata = base_path
                .metadata()
                .map_err(|_e| NewFileSystemCacheError::CouldntGetMetadata)?;
            if !metadata.is_dir() {
                return Err(NewFileSystemCacheError::ExistsButNoDirectory);
            }
            if metadata.permissions().readonly() {
                return Err(NewFileSystemCacheError::ReadonlyPath);
            }
        } else {
            // Create the directory and any parent directories if they don't yet exist.
            mkdir_p(&base_path).map_err(|_e| NewFileSystemCacheError::CouldntCreatePath)?;
        }

        Ok(Self {
            modules_path: modules_path(
                &base_path,
                current_wasmer_module_version(),
                &Target::default(),
            ),
            unchecked_modules,
        })
    }

    /// If `unchecked` is true, the cache will use the `*_unchecked` wasmer functions for
    /// loading modules from disk.
    pub fn set_module_unchecked(&mut self, unchecked: bool) {
        self.unchecked_modules = unchecked;
    }

    /// Returns the path to the serialized module with the given checksum.
    fn module_file(&self, checksum: &Checksum) -> PathBuf {
        let mut path = self.modules_path.clone();
        path.push(checksum.to_hex());
        path.set_extension("module");
        path
    }

    /// Loads a serialized module from the file system and returns a module (i.e. artifact + store),
    /// along with the size of the serialized module.
    pub fn load(
        &self,
        checksum: &Checksum,
        engine: &impl AsEngineRef,
    ) -> VmResult<Option<(Module, usize)>> {
        let file_path = self.module_file(checksum);

        let result = if self.unchecked_modules {
            unsafe { Module::deserialize_from_file_unchecked(engine, &file_path) }
        } else {
            unsafe { Module::deserialize_from_file(engine, &file_path) }
        };
        match result {
            Ok(module) => {
                let module_size = module_size(&file_path)?;
                Ok(Some((module, module_size)))
            }
            Err(DeserializeError::Io(err)) => match err.kind() {
                io::ErrorKind::NotFound => Ok(None),
                _ => Err(VmError::cache_err(format!(
                    "Error opening module file: {err}"
                ))),
            },
            Err(err) => Err(VmError::cache_err(format!(
                "Error deserializing module: {err}"
            ))),
        }
    }

    /// Stores a serialized module to the file system. Returns the size of the serialized module.
    pub fn store(&mut self, checksum: &Checksum, module: &Module) -> VmResult<usize> {
        mkdir_p(&self.modules_path)
            .map_err(|_e| VmError::cache_err("Error creating modules directory"))?;

        let path = self.module_file(checksum);
        module
            .serialize_to_file(&path)
            .map_err(|e| VmError::cache_err(format!("Error writing module to disk: {e}")))?;
        let module_size = module_size(&path)?;
        Ok(module_size)
    }

    /// Removes a serialized module from the file system.
    ///
    /// Returns true if the file existed and false if the file did not exist.
    pub fn remove(&mut self, checksum: &Checksum) -> VmResult<bool> {
        let file_path = self.module_file(checksum);

        if file_path.exists() {
            fs::remove_file(file_path)
                .map_err(|_e| VmError::cache_err("Error deleting module from disk"))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Returns the size of the module stored on disk
fn module_size(module_path: &Path) -> VmResult<usize> {
    let module_size: usize = module_path
        .metadata()
        .map_err(|_e| VmError::cache_err("Error getting file metadata"))? // ensure error message is not system specific
        .len()
        .try_into()
        .expect("Could not convert file size to usize");
    Ok(module_size)
}

/// Creates an identifier for the Wasmer `Target` that is used for
/// cache invalidation. The output is reasonable human friendly to be useable
/// in file path component.
fn target_id(target: &Target) -> String {
    // Use a custom Hasher implementation to avoid randomization.
    let mut deterministic_hasher = crc32fast::Hasher::new();
    target.hash(&mut deterministic_hasher);
    let hash = deterministic_hasher.finalize();
    format!("{}-{:08X}", target.triple(), hash) // print 4 byte hash as 8 hex characters
}

/// The path to the latest version of the modules.
fn modules_path(base_path: &Path, wasmer_module_version: u32, target: &Target) -> PathBuf {
    let version_dir = format!("{MODULE_SERIALIZATION_VERSION}-wasmer{wasmer_module_version}");
    let target_dir = target_id(target);
    base_path.join(version_dir).join(target_dir)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::{
        size::Size,
        wasm_backend::{compile, make_compiling_engine, make_runtime_engine},
    };
    use tempfile::TempDir;
    use wasmer::{imports, Instance as WasmerInstance, Store};
    use wasmer_middlewares::metering::set_remaining_points;

    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));
    const TESTING_GAS_LIMIT: u64 = 500_000_000;

    const SOME_WAT: &str = r#"(module
        (type $t0 (func (param i32) (result i32)))
        (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
            get_local $p0
            i32.const 1
            i32.add))
    "#;

    #[test]
    fn file_system_cache_run() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path(), false).unwrap() };

        // Create module
        let wasm = wat::parse_str(SOME_WAT).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Module does not exist
        let runtime_engine = make_runtime_engine(TESTING_MEMORY_LIMIT);
        let cached = cache.load(&checksum, &runtime_engine).unwrap();
        assert!(cached.is_none());

        // Store module
        let compiling_engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = compile(&compiling_engine, &wasm).unwrap();
        cache.store(&checksum, &module).unwrap();

        // Load module
        let cached = cache.load(&checksum, &runtime_engine).unwrap();
        assert!(cached.is_some());

        // Check the returned module is functional.
        // This is not really testing the cache API but better safe than sorry.
        {
            let (cached_module, module_size) = cached.unwrap();
            assert_eq!(module_size, module.serialize().unwrap().len());
            let import_object = imports! {};
            let mut store = Store::new(runtime_engine);
            let instance = WasmerInstance::new(&mut store, &cached_module, &import_object).unwrap();
            set_remaining_points(&mut store, &instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&mut store, &[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }

    #[test]
    fn file_system_cache_store_uses_expected_path() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path(), false).unwrap() };

        // Create module
        let wasm = wat::parse_str(SOME_WAT).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Store module
        let engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = compile(&engine, &wasm).unwrap();
        cache.store(&checksum, &module).unwrap();

        let mut globber = glob::glob(&format!(
            "{}/v8-wasmer5/**/{}.module",
            tmp_dir.path().to_string_lossy(),
            checksum
        ))
        .expect("Failed to read glob pattern");
        let file_path = globber.next().unwrap().unwrap();
        let _serialized_module = fs::read(file_path).unwrap();
    }

    #[test]
    fn file_system_cache_remove_works() {
        let tmp_dir = TempDir::new().unwrap();
        let mut cache = unsafe { FileSystemCache::new(tmp_dir.path(), false).unwrap() };

        // Create module
        let wasm = wat::parse_str(SOME_WAT).unwrap();
        let checksum = Checksum::generate(&wasm);

        // Store module
        let engine1 = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = compile(&engine1, &wasm).unwrap();
        cache.store(&checksum, &module).unwrap();

        // It's there
        let engine2 = make_runtime_engine(TESTING_MEMORY_LIMIT);
        assert!(cache.load(&checksum, &engine2).unwrap().is_some());

        // Remove module
        let existed = cache.remove(&checksum).unwrap();
        assert!(existed);

        // it's gone now
        assert!(cache.load(&checksum, &engine2).unwrap().is_none());

        // Remove again
        let existed = cache.remove(&checksum).unwrap();
        assert!(!existed);
    }

    #[test]
    fn target_id_works() {
        let triple = wasmer::Triple {
            architecture: wasmer::Architecture::X86_64,
            vendor: target_lexicon::Vendor::Nintendo,
            operating_system: target_lexicon::OperatingSystem::Fuchsia,
            environment: target_lexicon::Environment::Gnu,
            binary_format: target_lexicon::BinaryFormat::Coff,
        };
        let target = Target::new(triple.clone(), wasmer::CpuFeature::POPCNT.into());
        let id = target_id(&target);
        assert_eq!(id, "x86_64-nintendo-fuchsia-gnu-coff-01E9F9FE");
        // Changing CPU features changes the hash part
        let target = Target::new(triple, wasmer::CpuFeature::AVX512DQ.into());
        let id = target_id(&target);
        assert_eq!(id, "x86_64-nintendo-fuchsia-gnu-coff-93001945");

        // Works for durrect target (hashing is deterministic);
        let target = Target::default();
        let id1 = target_id(&target);
        let id2 = target_id(&target);
        assert_eq!(id1, id2);
    }

    #[test]
    fn modules_path_works() {
        let base = PathBuf::from("modules");
        let triple = wasmer::Triple {
            architecture: wasmer::Architecture::X86_64,
            vendor: target_lexicon::Vendor::Nintendo,
            operating_system: target_lexicon::OperatingSystem::Fuchsia,
            environment: target_lexicon::Environment::Gnu,
            binary_format: target_lexicon::BinaryFormat::Coff,
        };
        let target = Target::new(triple, wasmer::CpuFeature::POPCNT.into());
        let p = modules_path(&base, 17, &target);
        assert_eq!(
            p.as_os_str(),
            if cfg!(windows) {
                "modules\\v8-wasmer17\\x86_64-nintendo-fuchsia-gnu-coff-01E9F9FE"
            } else {
                "modules/v8-wasmer17/x86_64-nintendo-fuchsia-gnu-coff-01E9F9FE"
            }
        );
    }
}
