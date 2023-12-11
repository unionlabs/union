use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use wasmer::{Engine, Store};

use crate::backend::{Backend, BackendApi, Querier, Storage};
use crate::capabilities::required_capabilities_from_module;
use crate::checksum::Checksum;
use crate::compatibility::check_wasm;
use crate::errors::{VmError, VmResult};
use crate::filesystem::mkdir_p;
use crate::instance::{Instance, InstanceOptions};
use crate::modules::{CachedModule, FileSystemCache, InMemoryCache, PinnedMemoryCache};
use crate::parsed_wasm::ParsedWasm;
use crate::size::Size;
use crate::static_analysis::has_ibc_entry_points;
use crate::wasm_backend::{compile, make_compiling_engine, make_runtime_engine};

const STATE_DIR: &str = "state";
// Things related to the state of the blockchain.
const WASM_DIR: &str = "wasm";

const CACHE_DIR: &str = "cache";
// Cacheable things.
const MODULES_DIR: &str = "modules";

/// Statistics about the usage of a cache instance. Those values are node
/// specific and must not be used in a consensus critical context.
/// When a node is hit by a client for simulations or other queries, hits and misses
/// increase. Also a node restart will reset the values.
///
/// All values should be increment using saturated addition to ensure the node does not
/// crash in case the stats exceed the integer limit.
#[derive(Debug, Default, Clone, Copy)]
pub struct Stats {
    pub hits_pinned_memory_cache: u32,
    pub hits_memory_cache: u32,
    pub hits_fs_cache: u32,
    pub misses: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub stats: Stats,
    pub elements_pinned_memory_cache: usize,
    pub elements_memory_cache: usize,
    pub size_pinned_memory_cache: usize,
    pub size_memory_cache: usize,
}

#[derive(Clone, Debug)]
pub struct CacheOptions {
    /// The base directory of this cache.
    ///
    /// If this does not exist, it will be created. Not sure if this behaviour
    /// is desired but wasmd relies on it.
    pub base_dir: PathBuf,
    pub available_capabilities: HashSet<String>,
    pub memory_cache_size: Size,
    /// Memory limit for instances, in bytes. Use a value that is divisible by the Wasm page size 65536,
    /// e.g. full MiBs.
    pub instance_memory_limit: Size,
}

pub struct CacheInner {
    /// The directory in which the Wasm blobs are stored in the file system.
    wasm_path: PathBuf,
    pinned_memory_cache: PinnedMemoryCache,
    memory_cache: InMemoryCache,
    fs_cache: FileSystemCache,
    stats: Stats,
    /// A single engine to execute all contracts in this cache instance (usually
    /// this means all contracts in the process).
    ///
    /// This engine is headless, i.e. does not contain a Singlepass compiler.
    /// It only executes modules compiled with other engines.
    ///
    /// The engine has one memory limit set which is the same for all contracts
    /// running with it. If different memory limits would be needed for different
    /// contracts at some point, we'd need multiple engines. This is because the tunables
    /// that control the limit are attached to the engine.
    runtime_engine: Engine,
}

pub struct Cache<A: BackendApi, S: Storage, Q: Querier> {
    /// Available capabilities are immutable for the lifetime of the cache,
    /// i.e. any number of read-only references is allowed to access it concurrently.
    available_capabilities: HashSet<String>,
    inner: Mutex<CacheInner>,
    // Those two don't store data but only fix type information
    type_api: PhantomData<A>,
    type_storage: PhantomData<S>,
    type_querier: PhantomData<Q>,
    /// To prevent concurrent access to `WasmerInstance::new`
    instantiation_lock: Mutex<()>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct AnalysisReport {
    pub has_ibc_entry_points: bool,
    pub required_capabilities: HashSet<String>,
}

impl<A, S, Q> Cache<A, S, Q>
where
    A: BackendApi + 'static, // 'static is needed by `impl<…> Instance`
    S: Storage + 'static,    // 'static is needed by `impl<…> Instance`
    Q: Querier + 'static,    // 'static is needed by `impl<…> Instance`
{
    /// Creates a new cache that stores data in `base_dir`.
    ///
    /// # Safety
    ///
    /// This function is marked unsafe due to `FileSystemCache::new`, which implicitly
    /// assumes the disk contents are correct, and there's no way to ensure the artifacts
    /// stored in the cache haven't been corrupted or tampered with.
    pub unsafe fn new(options: CacheOptions) -> VmResult<Self> {
        let CacheOptions {
            base_dir,
            available_capabilities,
            memory_cache_size,
            instance_memory_limit,
        } = options;

        let state_path = base_dir.join(STATE_DIR);
        let cache_path = base_dir.join(CACHE_DIR);

        let wasm_path = state_path.join(WASM_DIR);

        // Ensure all the needed directories exist on disk.
        mkdir_p(&state_path).map_err(|_e| VmError::cache_err("Error creating state directory"))?;
        mkdir_p(&cache_path).map_err(|_e| VmError::cache_err("Error creating cache directory"))?;
        mkdir_p(&wasm_path).map_err(|_e| VmError::cache_err("Error creating wasm directory"))?;

        let fs_cache = FileSystemCache::new(cache_path.join(MODULES_DIR), false)
            .map_err(|e| VmError::cache_err(format!("Error file system cache: {e}")))?;
        Ok(Cache {
            available_capabilities,
            inner: Mutex::new(CacheInner {
                wasm_path,
                pinned_memory_cache: PinnedMemoryCache::new(),
                memory_cache: InMemoryCache::new(memory_cache_size),
                fs_cache,
                stats: Stats::default(),
                runtime_engine: make_runtime_engine(Some(instance_memory_limit)),
            }),
            type_storage: PhantomData::<S>,
            type_api: PhantomData::<A>,
            type_querier: PhantomData::<Q>,
            instantiation_lock: Mutex::new(()),
        })
    }

    /// If `unchecked` is true, the filesystem cache will use the `*_unchecked` wasmer functions for
    /// loading modules from disk.
    pub fn set_module_unchecked(&mut self, unchecked: bool) {
        self.inner
            .lock()
            .unwrap()
            .fs_cache
            .set_module_unchecked(unchecked);
    }

    pub fn stats(&self) -> Stats {
        self.inner.lock().unwrap().stats
    }

    pub fn metrics(&self) -> Metrics {
        let cache = self.inner.lock().unwrap();
        Metrics {
            stats: cache.stats,
            elements_pinned_memory_cache: cache.pinned_memory_cache.len(),
            elements_memory_cache: cache.memory_cache.len(),
            size_pinned_memory_cache: cache.pinned_memory_cache.size(),
            size_memory_cache: cache.memory_cache.size(),
        }
    }

    /// Takes a Wasm bytecode and stores it to the cache.
    ///
    /// This performs static checks, compiles the bytescode to a module and
    /// stores the Wasm file on disk.
    ///
    /// This does the same as [`save_wasm_unchecked`] plus the static checks.
    /// When a Wasm blob is stored the first time, use this function.
    pub fn save_wasm(&self, wasm: &[u8]) -> VmResult<Checksum> {
        check_wasm(wasm, &self.available_capabilities)?;
        self.save_wasm_unchecked(wasm)
    }

    /// Takes a Wasm bytecode and stores it to the cache.
    ///
    /// This compiles the bytescode to a module and
    /// stores the Wasm file on disk.
    ///
    /// This does the same as [`save_wasm`] but without the static checks.
    /// When a Wasm blob is stored which was previously checked (e.g. as part of state sync),
    /// use this function.
    pub fn save_wasm_unchecked(&self, wasm: &[u8]) -> VmResult<Checksum> {
        // We need a new engine for each Wasm -> module compilation due to the metering middleware.
        let compiling_engine = make_compiling_engine(None);
        // This module cannot be executed directly as it was not created with the runtime engine
        let module = compile(&compiling_engine, wasm)?;

        let mut cache = self.inner.lock().unwrap();
        let checksum = save_wasm_to_disk(&cache.wasm_path, wasm)?;
        cache.fs_cache.store(&checksum, &module)?;
        Ok(checksum)
    }

    /// Removes the Wasm blob for the given checksum from disk and its
    /// compiled module from the file system cache.
    ///
    /// The existence of the original code is required since the caller (wasmd)
    /// has to keep track of which entries we have here.
    pub fn remove_wasm(&self, checksum: &Checksum) -> VmResult<()> {
        let mut cache = self.inner.lock().unwrap();

        // Remove compiled moduled from disk (if it exists).
        // Here we could also delete from memory caches but this is not really
        // necessary as they are pushed out from the LRU over time or disappear
        // when the node process restarts.
        cache.fs_cache.remove(checksum)?;

        let path = &cache.wasm_path;
        remove_wasm_from_disk(path, checksum)?;
        Ok(())
    }

    /// Retrieves a Wasm blob that was previously stored via save_wasm.
    /// When the cache is instantiated with the same base dir, this finds Wasm files on disc across multiple cache instances (i.e. node restarts).
    /// This function is public to allow a checksum to Wasm lookup in the blockchain.
    ///
    /// If the given ID is not found or the content does not match the hash (=ID), an error is returned.
    pub fn load_wasm(&self, checksum: &Checksum) -> VmResult<Vec<u8>> {
        self.load_wasm_with_path(&self.inner.lock().unwrap().wasm_path, checksum)
    }

    fn load_wasm_with_path(&self, wasm_path: &Path, checksum: &Checksum) -> VmResult<Vec<u8>> {
        let code = load_wasm_from_disk(wasm_path, checksum)?;
        // verify hash matches (integrity check)
        if Checksum::generate(&code) != *checksum {
            Err(VmError::integrity_err())
        } else {
            Ok(code)
        }
    }

    /// Performs static anlyzation on this Wasm without compiling or instantiating it.
    ///
    /// Once the contract was stored via [`save_wasm`], this can be called at any point in time.
    /// It does not depend on any caching of the contract.
    pub fn analyze(&self, checksum: &Checksum) -> VmResult<AnalysisReport> {
        // Here we could use a streaming deserializer to slightly improve performance. However, this way it is DRYer.
        let wasm = self.load_wasm(checksum)?;
        let module = ParsedWasm::parse(&wasm)?;
        Ok(AnalysisReport {
            has_ibc_entry_points: has_ibc_entry_points(&module),
            required_capabilities: required_capabilities_from_module(&module),
        })
    }

    /// Pins a Module that was previously stored via save_wasm.
    ///
    /// The module is lookup first in the file system cache. If not found,
    /// the code is loaded from the file system, compiled, and stored into the
    /// pinned cache.
    ///
    /// If the given contract for the given checksum is not found, or the content
    /// does not match the checksum, an error is returned.
    pub fn pin(&self, checksum: &Checksum) -> VmResult<()> {
        let mut cache = self.inner.lock().unwrap();
        if cache.pinned_memory_cache.has(checksum) {
            return Ok(());
        }

        // We don't load from the memory cache because we had to create new store here and
        // serialize/deserialize the artifact to get a full clone. Could be done but adds some code
        // for a not-so-relevant use case.

        // Try to get module from file system cache
        if let Some((module, module_size)) = cache.fs_cache.load(checksum, &cache.runtime_engine)? {
            cache.stats.hits_fs_cache = cache.stats.hits_fs_cache.saturating_add(1);
            return cache
                .pinned_memory_cache
                .store(checksum, module, module_size);
        }

        // Re-compile from original Wasm bytecode
        let wasm = self.load_wasm_with_path(&cache.wasm_path, checksum)?;
        cache.stats.misses = cache.stats.misses.saturating_add(1);
        {
            // Module will run with a different engine, so we can set memory limit to None
            let compiling_engine = make_compiling_engine(None);
            // This module cannot be executed directly as it was not created with the runtime engine
            let module = compile(&compiling_engine, &wasm)?;
            cache.fs_cache.store(checksum, &module)?;
        }

        // This time we'll hit the file-system cache.
        let Some((module, module_size)) = cache.fs_cache.load(checksum, &cache.runtime_engine)?
        else {
            return Err(VmError::generic_err(
                "Can't load module from file system cache after storing it to file system cache (pin)",
            ));
        };

        cache
            .pinned_memory_cache
            .store(checksum, module, module_size)
    }

    /// Unpins a Module, i.e. removes it from the pinned memory cache.
    ///
    /// Not found IDs are silently ignored, and no integrity check (checksum validation) is done
    /// on the removed value.
    pub fn unpin(&self, checksum: &Checksum) -> VmResult<()> {
        self.inner
            .lock()
            .unwrap()
            .pinned_memory_cache
            .remove(checksum)
    }

    /// Returns an Instance tied to a previously saved Wasm.
    ///
    /// It takes a module from cache or Wasm code and instantiates it.
    pub fn get_instance(
        &self,
        checksum: &Checksum,
        backend: Backend<A, S, Q>,
        options: InstanceOptions,
    ) -> VmResult<Instance<A, S, Q>> {
        let (cached, store) = self.get_module(checksum)?;
        let instance = Instance::from_module(
            store,
            &cached.module,
            backend,
            options.gas_limit,
            options.print_debug,
            None,
            Some(&self.instantiation_lock),
        )?;
        Ok(instance)
    }

    /// Returns a module tied to a previously saved Wasm.
    /// Depending on availability, this is either generated from a memory cache, file system cache or Wasm code.
    /// This is part of `get_instance` but pulled out to reduce the locking time.
    fn get_module(&self, checksum: &Checksum) -> VmResult<(CachedModule, Store)> {
        let mut cache = self.inner.lock().unwrap();
        // Try to get module from the pinned memory cache
        if let Some(element) = cache.pinned_memory_cache.load(checksum)? {
            cache.stats.hits_pinned_memory_cache =
                cache.stats.hits_pinned_memory_cache.saturating_add(1);
            let store = Store::new(cache.runtime_engine.clone());
            return Ok((element, store));
        }

        // Get module from memory cache
        if let Some(element) = cache.memory_cache.load(checksum)? {
            cache.stats.hits_memory_cache = cache.stats.hits_memory_cache.saturating_add(1);
            let store = Store::new(cache.runtime_engine.clone());
            return Ok((element, store));
        }

        // Get module from file system cache
        if let Some((module, module_size)) = cache.fs_cache.load(checksum, &cache.runtime_engine)? {
            cache.stats.hits_fs_cache = cache.stats.hits_fs_cache.saturating_add(1);

            cache
                .memory_cache
                .store(checksum, module.clone(), module_size)?;
            let cached = CachedModule {
                module,
                size_estimate: module_size,
            };
            let store = Store::new(cache.runtime_engine.clone());
            return Ok((cached, store));
        }

        // Re-compile module from wasm
        //
        // This is needed for chains that upgrade their node software in a way that changes the module
        // serialization format. If you do not replay all transactions, previous calls of `save_wasm`
        // stored the old module format.
        let wasm = self.load_wasm_with_path(&cache.wasm_path, checksum)?;
        cache.stats.misses = cache.stats.misses.saturating_add(1);
        {
            // Module will run with a different engine, so we can set memory limit to None
            let compiling_engine = make_compiling_engine(None);
            // This module cannot be executed directly as it was not created with the runtime engine
            let module = compile(&compiling_engine, &wasm)?;
            cache.fs_cache.store(checksum, &module)?;
        }

        // This time we'll hit the file-system cache.
        let Some((module, module_size)) = cache.fs_cache.load(checksum, &cache.runtime_engine)?
        else {
            return Err(VmError::generic_err(
                "Can't load module from file system cache after storing it to file system cache (get_module)",
            ));
        };
        cache
            .memory_cache
            .store(checksum, module.clone(), module_size)?;
        let cached = CachedModule {
            module,
            size_estimate: module_size,
        };
        let store = Store::new(cache.runtime_engine.clone());
        Ok((cached, store))
    }
}

unsafe impl<A, S, Q> Sync for Cache<A, S, Q>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
}

unsafe impl<A, S, Q> Send for Cache<A, S, Q>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
}

/// save stores the wasm code in the given directory and returns an ID for lookup.
/// It will create the directory if it doesn't exist.
/// Saving the same byte code multiple times is allowed.
fn save_wasm_to_disk(dir: impl Into<PathBuf>, wasm: &[u8]) -> VmResult<Checksum> {
    // calculate filename
    let checksum = Checksum::generate(wasm);
    let filename = checksum.to_hex();
    let filepath = dir.into().join(filename).with_extension("wasm");

    // write data to file
    // Since the same filename (a collision resistent hash) cannot be generated from two different byte codes
    // (even if a malicious actor tried), it is safe to override.
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(filepath)
        .map_err(|e| VmError::cache_err(format!("Error opening Wasm file for writing: {e}")))?;
    file.write_all(wasm)
        .map_err(|e| VmError::cache_err(format!("Error writing Wasm file: {e}")))?;

    Ok(checksum)
}

fn load_wasm_from_disk(dir: impl Into<PathBuf>, checksum: &Checksum) -> VmResult<Vec<u8>> {
    // this requires the directory and file to exist
    // The files previously had no extension, so to allow for a smooth transition,
    // we also try to load the file without the wasm extension.
    let path = dir.into().join(checksum.to_hex());
    let mut file = File::open(path.with_extension("wasm"))
        .or_else(|_| File::open(path))
        .map_err(|_e| VmError::cache_err("Error opening Wasm file for reading"))?;

    let mut wasm = Vec::<u8>::new();
    file.read_to_end(&mut wasm)
        .map_err(|_e| VmError::cache_err("Error reading Wasm file"))?;
    Ok(wasm)
}

/// Removes the Wasm blob for the given checksum from disk.
///
/// In contrast to the file system cache, the existence of the original
/// code is required. So a non-existent file leads to an error as it
/// indicates a bug.
fn remove_wasm_from_disk(dir: impl Into<PathBuf>, checksum: &Checksum) -> VmResult<()> {
    // the files previously had no extension, so to allow for a smooth transition, we delete both
    let path = dir.into().join(checksum.to_hex());
    let wasm_path = path.with_extension("wasm");

    let path_exists = path.exists();
    let wasm_path_exists = wasm_path.exists();
    if !path_exists && !wasm_path_exists {
        return Err(VmError::cache_err("Wasm file does not exist"));
    }

    if path_exists {
        fs::remove_file(path)
            .map_err(|_e| VmError::cache_err("Error removing Wasm file from disk"))?;
    }

    if wasm_path_exists {
        fs::remove_file(wasm_path)
            .map_err(|_e| VmError::cache_err("Error removing Wasm file from disk"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calls::{call_execute, call_instantiate};
    use crate::capabilities::capabilities_from_csv;
    use crate::errors::VmError;
    use crate::testing::{mock_backend, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, Empty};
    use std::fs::{create_dir_all, remove_dir_all, OpenOptions};
    use std::io::Write;
    use tempfile::TempDir;

    const TESTING_GAS_LIMIT: u64 = 500_000_000_000; // ~0.5ms
    const TESTING_MEMORY_LIMIT: Size = Size::mebi(16);
    const TESTING_OPTIONS: InstanceOptions = InstanceOptions {
        gas_limit: TESTING_GAS_LIMIT,
        print_debug: false,
    };
    const TESTING_MEMORY_CACHE_SIZE: Size = Size::mebi(200);

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static IBC_CONTRACT: &[u8] = include_bytes!("../testdata/ibc_reflect.wasm");
    // Invalid because it doesn't contain required memory and exports
    static INVALID_CONTRACT_WAT: &str = r#"(module
        (type $t0 (func (param i32) (result i32)))
        (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
            get_local $p0
            i32.const 1
            i32.add))
    "#;

    fn default_capabilities() -> HashSet<String> {
        capabilities_from_csv("iterator,staking")
    }

    fn make_testing_options() -> CacheOptions {
        CacheOptions {
            base_dir: TempDir::new().unwrap().into_path(),
            available_capabilities: default_capabilities(),
            memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
            instance_memory_limit: TESTING_MEMORY_LIMIT,
        }
    }

    fn make_stargate_testing_options() -> CacheOptions {
        let mut capabilities = default_capabilities();
        capabilities.insert("stargate".into());
        CacheOptions {
            base_dir: TempDir::new().unwrap().into_path(),
            available_capabilities: capabilities,
            memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
            instance_memory_limit: TESTING_MEMORY_LIMIT,
        }
    }

    /// Takes an instance and executes it
    fn test_hackatom_instance_execution<A, S, Q>(instance: &mut Instance<A, S, Q>)
    where
        A: BackendApi + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
    {
        // instantiate
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let response = call_instantiate::<_, _, _, Empty>(instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert_eq!(response.messages.len(), 0);

        // execute
        let info = mock_info("verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let response = call_execute::<_, _, _, Empty>(instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert_eq!(response.messages.len(), 1);
    }

    #[test]
    fn new_base_dir_will_be_created() {
        let my_base_dir = TempDir::new()
            .unwrap()
            .into_path()
            .join("non-existent-sub-dir");
        let options = CacheOptions {
            base_dir: my_base_dir.clone(),
            ..make_testing_options()
        };
        assert!(!my_base_dir.is_dir());
        let _cache = unsafe { Cache::<MockApi, MockStorage, MockQuerier>::new(options).unwrap() };
        assert!(my_base_dir.is_dir());
    }

    #[test]
    fn save_wasm_works() {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        cache.save_wasm(CONTRACT).unwrap();
    }

    #[test]
    // This property is required when the same bytecode is uploaded multiple times
    fn save_wasm_allows_saving_multiple_times() {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        cache.save_wasm(CONTRACT).unwrap();
        cache.save_wasm(CONTRACT).unwrap();
    }

    #[test]
    fn save_wasm_rejects_invalid_contract() {
        let wasm = wat::parse_str(INVALID_CONTRACT_WAT).unwrap();

        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        let save_result = cache.save_wasm(&wasm);
        match save_result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(msg, "Wasm contract must contain exactly one memory")
            }
            e => panic!("Unexpected error {e:?}"),
        }
    }

    #[test]
    fn save_wasm_fills_file_system_but_not_memory_cache() {
        // Who knows if and when the uploaded contract will be executed. Don't pollute
        // memory cache before the init call.

        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        let backend = mock_backend(&[]);
        let _ = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn save_wasm_unchecked_works() {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        cache.save_wasm_unchecked(CONTRACT).unwrap();
    }

    #[test]
    fn save_wasm_unchecked_accepts_invalid_contract() {
        let wasm = wat::parse_str(INVALID_CONTRACT_WAT).unwrap();

        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        cache.save_wasm_unchecked(&wasm).unwrap();
    }

    #[test]
    fn load_wasm_works() {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        let restored = cache.load_wasm(&checksum).unwrap();
        assert_eq!(restored, CONTRACT);
    }

    #[test]
    fn load_wasm_works_across_multiple_cache_instances() {
        let tmp_dir = TempDir::new().unwrap();
        let id: Checksum;

        {
            let options1 = CacheOptions {
                base_dir: tmp_dir.path().to_path_buf(),
                available_capabilities: default_capabilities(),
                memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
                instance_memory_limit: TESTING_MEMORY_LIMIT,
            };
            let cache1: Cache<MockApi, MockStorage, MockQuerier> =
                unsafe { Cache::new(options1).unwrap() };
            id = cache1.save_wasm(CONTRACT).unwrap();
        }

        {
            let options2 = CacheOptions {
                base_dir: tmp_dir.path().to_path_buf(),
                available_capabilities: default_capabilities(),
                memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
                instance_memory_limit: TESTING_MEMORY_LIMIT,
            };
            let cache2: Cache<MockApi, MockStorage, MockQuerier> =
                unsafe { Cache::new(options2).unwrap() };
            let restored = cache2.load_wasm(&id).unwrap();
            assert_eq!(restored, CONTRACT);
        }
    }

    #[test]
    fn load_wasm_errors_for_non_existent_id() {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = Checksum::from([
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5,
        ]);

        match cache.load_wasm(&checksum).unwrap_err() {
            VmError::CacheErr { msg, .. } => {
                assert_eq!(msg, "Error opening Wasm file for reading")
            }
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn load_wasm_errors_for_corrupted_wasm() {
        let tmp_dir = TempDir::new().unwrap();
        let options = CacheOptions {
            base_dir: tmp_dir.path().to_path_buf(),
            available_capabilities: default_capabilities(),
            memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
            instance_memory_limit: TESTING_MEMORY_LIMIT,
        };
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // Corrupt cache file
        let filepath = tmp_dir
            .path()
            .join(STATE_DIR)
            .join(WASM_DIR)
            .join(checksum.to_hex())
            .with_extension("wasm");
        let mut file = OpenOptions::new().write(true).open(filepath).unwrap();
        file.write_all(b"broken data").unwrap();

        let res = cache.load_wasm(&checksum);
        match res {
            Err(VmError::IntegrityErr { .. }) => {}
            Err(e) => panic!("Unexpected error: {e:?}"),
            Ok(_) => panic!("This must not succeed"),
        }
    }

    #[test]
    fn remove_wasm_works() {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_testing_options()).unwrap() };

        // Store
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // Exists
        cache.load_wasm(&checksum).unwrap();

        // Remove
        cache.remove_wasm(&checksum).unwrap();

        // Does not exist anymore
        match cache.load_wasm(&checksum).unwrap_err() {
            VmError::CacheErr { msg, .. } => {
                assert_eq!(msg, "Error opening Wasm file for reading")
            }
            e => panic!("Unexpected error: {e:?}"),
        }

        // Removing again fails
        match cache.remove_wasm(&checksum).unwrap_err() {
            VmError::CacheErr { msg, .. } => {
                assert_eq!(msg, "Wasm file does not exist")
            }
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn get_instance_finds_cached_module() {
        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();
        let backend = mock_backend(&[]);
        let _instance = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn get_instance_finds_cached_modules_and_stores_to_memory() {
        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();
        let backend1 = mock_backend(&[]);
        let backend2 = mock_backend(&[]);
        let backend3 = mock_backend(&[]);
        let backend4 = mock_backend(&[]);
        let backend5 = mock_backend(&[]);

        // from file system
        let _instance1 = cache
            .get_instance(&checksum, backend1, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // from memory
        let _instance2 = cache
            .get_instance(&checksum, backend2, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // from memory again
        let _instance3 = cache
            .get_instance(&checksum, backend3, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 2);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // pinning hits the file system cache
        cache.pin(&checksum).unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 2);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);

        // from pinned memory cache
        let _instance4 = cache
            .get_instance(&checksum, backend4, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 1);
        assert_eq!(cache.stats().hits_memory_cache, 2);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);

        // from pinned memory cache again
        let _instance5 = cache
            .get_instance(&checksum, backend5, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 2);
        assert_eq!(cache.stats().hits_memory_cache, 2);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn get_instance_recompiles_module() {
        let options = make_testing_options();
        let cache = unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // Remove compiled module from disk
        remove_dir_all(options.base_dir.join(CACHE_DIR).join(MODULES_DIR)).unwrap();

        // The first get_instance recompiles the Wasm (miss)
        let backend = mock_backend(&[]);
        let _instance = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 0);
        assert_eq!(cache.stats().misses, 1);

        // The second get_instance finds the module in cache (hit)
        let backend = mock_backend(&[]);
        let _instance = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 0);
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn call_instantiate_on_cached_contract() {
        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // from file system
        {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            // instantiate
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let res =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            let msgs = res.unwrap().messages;
            assert_eq!(msgs.len(), 0);
        }

        // from memory
        {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert_eq!(cache.stats().hits_memory_cache, 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            // instantiate
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let res =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            let msgs = res.unwrap().messages;
            assert_eq!(msgs.len(), 0);
        }

        // from pinned memory
        {
            cache.pin(&checksum).unwrap();

            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 1);
            assert_eq!(cache.stats().hits_memory_cache, 1);
            assert_eq!(cache.stats().hits_fs_cache, 2);
            assert_eq!(cache.stats().misses, 0);

            // instantiate
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let res =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            let msgs = res.unwrap().messages;
            assert_eq!(msgs.len(), 0);
        }
    }

    #[test]
    fn call_execute_on_cached_contract() {
        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // from file system
        {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            // instantiate
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let response =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                    .unwrap()
                    .unwrap();
            assert_eq!(response.messages.len(), 0);

            // execute
            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let response = call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.messages.len(), 1);
        }

        // from memory
        {
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert_eq!(cache.stats().hits_memory_cache, 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);

            // instantiate
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let response =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                    .unwrap()
                    .unwrap();
            assert_eq!(response.messages.len(), 0);

            // execute
            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let response = call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.messages.len(), 1);
        }

        // from pinned memory
        {
            cache.pin(&checksum).unwrap();

            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), TESTING_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 1);
            assert_eq!(cache.stats().hits_memory_cache, 1);
            assert_eq!(cache.stats().hits_fs_cache, 2);
            assert_eq!(cache.stats().misses, 0);

            // instantiate
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let response =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                    .unwrap()
                    .unwrap();
            assert_eq!(response.messages.len(), 0);

            // execute
            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let response = call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            assert_eq!(response.messages.len(), 1);
        }
    }

    #[test]
    fn call_execute_on_recompiled_contract() {
        let options = make_testing_options();
        let cache = unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // Remove compiled module from disk
        remove_dir_all(options.base_dir.join(CACHE_DIR).join(MODULES_DIR)).unwrap();

        // Recompiles the Wasm (miss on all caches)
        let backend = mock_backend(&[]);
        let mut instance = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 0);
        assert_eq!(cache.stats().misses, 1);
        test_hackatom_instance_execution(&mut instance);
    }

    #[test]
    fn use_multiple_cached_instances_of_same_contract() {
        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // these differentiate the two instances of the same contract
        let backend1 = mock_backend(&[]);
        let backend2 = mock_backend(&[]);

        // instantiate instance 1
        let mut instance = cache
            .get_instance(&checksum, backend1, TESTING_OPTIONS)
            .unwrap();
        let info = mock_info("owner1", &coins(1000, "earth"));
        let msg = br#"{"verifier": "sue", "beneficiary": "mary"}"#;
        let res =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let backend1 = instance.recycle().unwrap();

        // instantiate instance 2
        let mut instance = cache
            .get_instance(&checksum, backend2, TESTING_OPTIONS)
            .unwrap();
        let info = mock_info("owner2", &coins(500, "earth"));
        let msg = br#"{"verifier": "bob", "beneficiary": "john"}"#;
        let res =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(msgs.len(), 0);
        let backend2 = instance.recycle().unwrap();

        // run contract 2 - just sanity check - results validate in contract unit tests
        let mut instance = cache
            .get_instance(&checksum, backend2, TESTING_OPTIONS)
            .unwrap();
        let info = mock_info("bob", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let res = call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());

        // run contract 1 - just sanity check - results validate in contract unit tests
        let mut instance = cache
            .get_instance(&checksum, backend1, TESTING_OPTIONS)
            .unwrap();
        let info = mock_info("sue", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        let res = call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        let msgs = res.unwrap().messages;
        assert_eq!(1, msgs.len());
    }

    #[test]
    fn resets_gas_when_reusing_instance() {
        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        let backend1 = mock_backend(&[]);
        let backend2 = mock_backend(&[]);

        // Init from module cache
        let mut instance1 = cache
            .get_instance(&checksum, backend1, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
        let original_gas = instance1.get_gas_left();

        // Consume some gas
        let info = mock_info("owner1", &coins(1000, "earth"));
        let msg = br#"{"verifier": "sue", "beneficiary": "mary"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance1, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
        assert!(instance1.get_gas_left() < original_gas);

        // Init from memory cache
        let mut instance2 = cache
            .get_instance(&checksum, backend2, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
        assert_eq!(instance2.get_gas_left(), TESTING_GAS_LIMIT);
    }

    #[test]
    fn recovers_from_out_of_gas() {
        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        let backend1 = mock_backend(&[]);
        let backend2 = mock_backend(&[]);

        // Init from module cache
        let options = InstanceOptions {
            gas_limit: 10,
            print_debug: false,
        };
        let mut instance1 = cache.get_instance(&checksum, backend1, options).unwrap();
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);

        // Consume some gas. This fails
        let info1 = mock_info("owner1", &coins(1000, "earth"));
        let msg1 = br#"{"verifier": "sue", "beneficiary": "mary"}"#;
        match call_instantiate::<_, _, _, Empty>(&mut instance1, &mock_env(), &info1, msg1)
            .unwrap_err()
        {
            VmError::GasDepletion { .. } => (), // all good, continue
            e => panic!("unexpected error, {e:?}"),
        }
        assert_eq!(instance1.get_gas_left(), 0);

        // Init from memory cache
        let options = InstanceOptions {
            gas_limit: TESTING_GAS_LIMIT,
            print_debug: false,
        };
        let mut instance2 = cache.get_instance(&checksum, backend2, options).unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
        assert_eq!(instance2.get_gas_left(), TESTING_GAS_LIMIT);

        // Now it works
        let info2 = mock_info("owner2", &coins(500, "earth"));
        let msg2 = br#"{"verifier": "bob", "beneficiary": "john"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance2, &mock_env(), &info2, msg2)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn save_wasm_to_disk_works_for_same_data_multiple_times() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let code = vec![12u8; 17];

        save_wasm_to_disk(path, &code).unwrap();
        save_wasm_to_disk(path, &code).unwrap();
    }

    #[test]
    fn save_wasm_to_disk_fails_on_non_existent_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("something");
        let code = vec![12u8; 17];
        let res = save_wasm_to_disk(path.to_str().unwrap(), &code);
        assert!(res.is_err());
    }

    #[test]
    fn load_wasm_from_disk_works() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let code = vec![12u8; 17];
        let checksum = save_wasm_to_disk(path, &code).unwrap();

        let loaded = load_wasm_from_disk(path, &checksum).unwrap();
        assert_eq!(code, loaded);
    }

    #[test]
    fn load_wasm_from_disk_works_in_subfolder() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("something");
        create_dir_all(&path).unwrap();
        let code = vec![12u8; 17];
        let checksum = save_wasm_to_disk(&path, &code).unwrap();

        let loaded = load_wasm_from_disk(&path, &checksum).unwrap();
        assert_eq!(code, loaded);
    }

    #[test]
    fn remove_wasm_from_disk_works() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path();
        let code = vec![12u8; 17];
        let checksum = save_wasm_to_disk(path, &code).unwrap();

        remove_wasm_from_disk(path, &checksum).unwrap();

        // removing again fails

        match remove_wasm_from_disk(path, &checksum).unwrap_err() {
            VmError::CacheErr { msg } => assert_eq!(msg, "Wasm file does not exist"),
            err => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn analyze_works() {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(make_stargate_testing_options()).unwrap() };

        let checksum1 = cache.save_wasm(CONTRACT).unwrap();
        let report1 = cache.analyze(&checksum1).unwrap();
        assert_eq!(
            report1,
            AnalysisReport {
                has_ibc_entry_points: false,
                required_capabilities: HashSet::new(),
            }
        );

        let checksum2 = cache.save_wasm(IBC_CONTRACT).unwrap();
        let report2 = cache.analyze(&checksum2).unwrap();
        assert_eq!(
            report2,
            AnalysisReport {
                has_ibc_entry_points: true,
                required_capabilities: HashSet::from_iter([
                    "iterator".to_string(),
                    "stargate".to_string()
                ]),
            }
        );
    }

    #[test]
    fn pin_unpin_works() {
        let cache = unsafe { Cache::new(make_testing_options()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // check not pinned
        let backend = mock_backend(&[]);
        let mut instance = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 1);
        assert_eq!(cache.stats().misses, 0);
        test_hackatom_instance_execution(&mut instance);

        // first pin hits file system cache
        cache.pin(&checksum).unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);

        // consecutive pins are no-ops
        cache.pin(&checksum).unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);

        // check pinned
        let backend = mock_backend(&[]);
        let mut instance = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 1);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);
        test_hackatom_instance_execution(&mut instance);

        // unpin
        cache.unpin(&checksum).unwrap();

        // verify unpinned
        let backend = mock_backend(&[]);
        let mut instance = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 1);
        assert_eq!(cache.stats().hits_memory_cache, 1);
        assert_eq!(cache.stats().hits_fs_cache, 2);
        assert_eq!(cache.stats().misses, 0);
        test_hackatom_instance_execution(&mut instance);

        // unpin again has no effect
        cache.unpin(&checksum).unwrap();

        // unpin non existent id has no effect
        let non_id = Checksum::generate(b"non_existent");
        cache.unpin(&non_id).unwrap();
    }

    #[test]
    fn pin_recompiles_module() {
        let options = make_testing_options();
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // Remove compiled module from disk
        remove_dir_all(options.base_dir.join(CACHE_DIR).join(MODULES_DIR)).unwrap();

        // Pin misses, forcing a re-compile of the module
        cache.pin(&checksum).unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 0);
        assert_eq!(cache.stats().misses, 1);

        // After the compilation in pin, the module can be used from pinned memory cache
        let backend = mock_backend(&[]);
        let mut instance = cache
            .get_instance(&checksum, backend, TESTING_OPTIONS)
            .unwrap();
        assert_eq!(cache.stats().hits_pinned_memory_cache, 1);
        assert_eq!(cache.stats().hits_memory_cache, 0);
        assert_eq!(cache.stats().hits_fs_cache, 0);
        assert_eq!(cache.stats().misses, 1);
        test_hackatom_instance_execution(&mut instance);
    }

    #[test]
    fn loading_without_extension_works() {
        let tmp_dir = TempDir::new().unwrap();
        let options = CacheOptions {
            base_dir: tmp_dir.path().to_path_buf(),
            available_capabilities: default_capabilities(),
            memory_cache_size: TESTING_MEMORY_CACHE_SIZE,
            instance_memory_limit: TESTING_MEMORY_LIMIT,
        };
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        // Move the saved wasm to the old path (without extension)
        let old_path = tmp_dir
            .path()
            .join(STATE_DIR)
            .join(WASM_DIR)
            .join(checksum.to_hex());
        let new_path = old_path.with_extension("wasm");
        fs::rename(new_path, old_path).unwrap();

        // loading wasm from before the wasm extension was added should still work
        let restored = cache.load_wasm(&checksum).unwrap();
        assert_eq!(restored, CONTRACT);
    }
}
