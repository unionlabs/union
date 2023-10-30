use clru::{CLruCache, CLruCacheConfig, WeightScale};
use std::collections::hash_map::RandomState;
use std::num::NonZeroUsize;
use wasmer::Module;

use super::sized_module::SizedModule;
use crate::{Checksum, Size, VmError, VmResult};

// Minimum module size.
// Based on `examples/module_size.sh`, and the cosmwasm-plus contracts.
// We use an estimated *minimum* module size in order to compute a number of pre-allocated entries
// that are enough to handle a size-limited cache without requiring re-allocation / resizing.
// This will incurr an extra memory cost for the unused entries, but it's negligible:
// Assuming the cost per entry is 48 bytes, 10000 entries will have an extra cost of just ~500 kB.
// Which is a very small percentage (~0.03%) of our typical cache memory budget (2 GB).
const MINIMUM_MODULE_SIZE: Size = Size::kibi(250);

#[derive(Debug)]
struct SizeScale;

impl WeightScale<Checksum, SizedModule> for SizeScale {
    #[inline]
    fn weight(&self, _key: &Checksum, value: &SizedModule) -> usize {
        value.size
    }
}

/// An in-memory module cache
pub struct InMemoryCache {
    modules: Option<CLruCache<Checksum, SizedModule, RandomState, SizeScale>>,
}

impl InMemoryCache {
    /// Creates a new cache with the given size (in bytes)
    /// and pre-allocated entries.
    pub fn new(size: Size) -> Self {
        let preallocated_entries = size.0 / MINIMUM_MODULE_SIZE.0;

        InMemoryCache {
            modules: if size.0 > 0 {
                Some(CLruCache::with_config(
                    CLruCacheConfig::new(NonZeroUsize::new(size.0).unwrap())
                        .with_memory(preallocated_entries)
                        .with_scale(SizeScale),
                ))
            } else {
                None
            },
        }
    }

    pub fn store(&mut self, checksum: &Checksum, module: Module, size: usize) -> VmResult<()> {
        if let Some(modules) = &mut self.modules {
            modules
                .put_with_weight(*checksum, SizedModule { module, size })
                .map_err(|e| VmError::cache_err(format!("{:?}", e)))?;
        }
        Ok(())
    }

    /// Looks up a module in the cache and creates a new module
    pub fn load(&mut self, checksum: &Checksum) -> VmResult<Option<SizedModule>> {
        if let Some(modules) = &mut self.modules {
            match modules.get(checksum) {
                Some(module) => Ok(Some(module.clone())),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Returns the number of elements in the cache.
    pub fn len(&self) -> usize {
        self.modules
            .as_ref()
            .map(|modules| modules.len())
            .unwrap_or_default()
    }

    /// Returns cumulative size of all elements in the cache.
    ///
    /// This is based on the values provided with `store`. No actual
    /// memory size is measured here.
    pub fn size(&self) -> usize {
        self.modules
            .as_ref()
            .map(|modules| modules.weight())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::size::Size;
    use crate::wasm_backend::compile;
    use std::mem;
    use wasmer::{imports, Instance as WasmerInstance};
    use wasmer_middlewares::metering::set_remaining_points;

    const TESTING_GAS_LIMIT: u64 = 500_000_000;
    // Based on `examples/module_size.sh`
    const TESTING_WASM_SIZE_FACTOR: usize = 18;

    #[test]
    fn check_element_sizes() {
        let key_size = mem::size_of::<Checksum>();
        assert_eq!(key_size, 32);

        // A Module consists of a Store (3 Arcs) and an Arc to the Engine.
        // This is 3 * 64bit of data, but we don't get any guarantee how the Rust structs
        // Module and Store are aligned (https://doc.rust-lang.org/reference/type-layout.html#the-default-representation).
        // So we get this value by trial and error. It can change over time and across platforms.
        let value_size = mem::size_of::<Module>();
        assert_eq!(value_size, 56);

        // Just in case we want to go that route
        let boxed_value_size = mem::size_of::<Box<Module>>();
        assert_eq!(boxed_value_size, 8);
    }

    #[test]
    fn in_memory_cache_run() {
        let mut cache = InMemoryCache::new(Size::mebi(200));

        // Create module
        let wasm = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add)
            )"#,
        )
        .unwrap();
        let checksum = Checksum::generate(&wasm);

        // Module does not exist
        let cache_entry = cache.load(&checksum).unwrap();
        assert!(cache_entry.is_none());

        // Compile module
        let original = compile(&wasm, None, &[]).unwrap();

        // Ensure original module can be executed
        {
            let instance = WasmerInstance::new(&original, &imports! {}).unwrap();
            set_remaining_points(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }

        // Store module
        let size = wasm.len() * TESTING_WASM_SIZE_FACTOR;
        cache.store(&checksum, original, size).unwrap();

        // Load module
        let cached = cache.load(&checksum).unwrap().unwrap();

        // Ensure cached module can be executed
        {
            let instance = WasmerInstance::new(&cached.module, &imports! {}).unwrap();
            set_remaining_points(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }

    #[test]
    fn len_works() {
        let mut cache = InMemoryCache::new(Size::mebi(2));

        // Create module
        let wasm1 = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add)
            )"#,
        )
        .unwrap();
        let checksum1 = Checksum::generate(&wasm1);
        let wasm2 = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_two") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 2
                i32.add)
            )"#,
        )
        .unwrap();
        let checksum2 = Checksum::generate(&wasm2);
        let wasm3 = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_three") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 3
                i32.add)
            )"#,
        )
        .unwrap();
        let checksum3 = Checksum::generate(&wasm3);

        assert_eq!(cache.len(), 0);

        // Add 1
        cache
            .store(&checksum1, compile(&wasm1, None, &[]).unwrap(), 900_000)
            .unwrap();
        assert_eq!(cache.len(), 1);

        // Add 2
        cache
            .store(&checksum2, compile(&wasm2, None, &[]).unwrap(), 900_000)
            .unwrap();
        assert_eq!(cache.len(), 2);

        // Add 3 (pushes out the previous two)
        cache
            .store(&checksum3, compile(&wasm3, None, &[]).unwrap(), 1_500_000)
            .unwrap();
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn size_works() {
        let mut cache = InMemoryCache::new(Size::mebi(2));

        // Create module
        let wasm1 = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add)
            )"#,
        )
        .unwrap();
        let checksum1 = Checksum::generate(&wasm1);
        let wasm2 = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_two") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 2
                i32.add)
            )"#,
        )
        .unwrap();
        let checksum2 = Checksum::generate(&wasm2);
        let wasm3 = wat::parse_str(
            r#"(module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_three") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 3
                i32.add)
            )"#,
        )
        .unwrap();
        let checksum3 = Checksum::generate(&wasm3);

        assert_eq!(cache.size(), 0);

        // Add 1
        cache
            .store(&checksum1, compile(&wasm1, None, &[]).unwrap(), 900_000)
            .unwrap();
        assert_eq!(cache.size(), 900_000);

        // Add 2
        cache
            .store(&checksum2, compile(&wasm2, None, &[]).unwrap(), 800_000)
            .unwrap();
        assert_eq!(cache.size(), 1_700_000);

        // Add 3 (pushes out the previous two)
        cache
            .store(&checksum3, compile(&wasm3, None, &[]).unwrap(), 1_500_000)
            .unwrap();
        assert_eq!(cache.size(), 1_500_000);
    }
}
