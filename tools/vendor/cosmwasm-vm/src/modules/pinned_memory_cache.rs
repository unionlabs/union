use std::collections::HashMap;
use wasmer::Module;

use super::sized_module::SizedModule;
use crate::{Checksum, VmResult};

/// An pinned in memory module cache
pub struct PinnedMemoryCache {
    modules: HashMap<Checksum, SizedModule>,
}

impl PinnedMemoryCache {
    /// Creates a new cache
    pub fn new() -> Self {
        PinnedMemoryCache {
            modules: HashMap::new(),
        }
    }

    pub fn store(&mut self, checksum: &Checksum, module: Module, size: usize) -> VmResult<()> {
        self.modules.insert(*checksum, SizedModule { module, size });
        Ok(())
    }

    /// Removes a module from the cache
    /// Not found modules are silently ignored. Potential integrity errors (wrong checksum) are not checked / enforced
    pub fn remove(&mut self, checksum: &Checksum) -> VmResult<()> {
        self.modules.remove(checksum);
        Ok(())
    }

    /// Looks up a module in the cache and creates a new module
    pub fn load(&mut self, checksum: &Checksum) -> VmResult<Option<Module>> {
        match self.modules.get(checksum) {
            Some(module) => Ok(Some(module.module.clone())),
            None => Ok(None),
        }
    }

    /// Returns true if and only if this cache has an entry identified by the given checksum
    pub fn has(&self, checksum: &Checksum) -> bool {
        self.modules.contains_key(checksum)
    }

    /// Returns the number of elements in the cache.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Returns cumulative size of all elements in the cache.
    ///
    /// This is based on the values provided with `store`. No actual
    /// memory size is measured here.
    pub fn size(&self) -> usize {
        self.modules.values().map(|module| module.size).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_backend::compile;
    use wasmer::{imports, Instance as WasmerInstance};
    use wasmer_middlewares::metering::set_remaining_points;

    const TESTING_GAS_LIMIT: u64 = 500_000_000;

    #[test]
    fn pinned_memory_cache_run() {
        let mut cache = PinnedMemoryCache::new();

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
        cache.store(&checksum, original, 0).unwrap();

        // Load module
        let cached = cache.load(&checksum).unwrap().unwrap();

        // Ensure cached module can be executed
        {
            let instance = WasmerInstance::new(&cached, &imports! {}).unwrap();
            set_remaining_points(&instance, TESTING_GAS_LIMIT);
            let add_one = instance.exports.get_function("add_one").unwrap();
            let result = add_one.call(&[42.into()]).unwrap();
            assert_eq!(result[0].unwrap_i32(), 43);
        }
    }

    #[test]
    fn has_works() {
        let mut cache = PinnedMemoryCache::new();

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

        assert!(!cache.has(&checksum));

        // Add
        let original = compile(&wasm, None, &[]).unwrap();
        cache.store(&checksum, original, 0).unwrap();

        assert!(cache.has(&checksum));

        // Remove
        cache.remove(&checksum).unwrap();

        assert!(!cache.has(&checksum));
    }

    #[test]
    fn len_works() {
        let mut cache = PinnedMemoryCache::new();

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

        assert_eq!(cache.len(), 0);

        // Add
        let original = compile(&wasm, None, &[]).unwrap();
        cache.store(&checksum, original, 0).unwrap();

        assert_eq!(cache.len(), 1);

        // Remove
        cache.remove(&checksum).unwrap();

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn size_works() {
        let mut cache = PinnedMemoryCache::new();

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

        assert_eq!(cache.size(), 0);

        // Add 1
        let original = compile(&wasm1, None, &[]).unwrap();
        cache.store(&checksum1, original, 500).unwrap();
        assert_eq!(cache.size(), 500);

        // Add 2
        let original = compile(&wasm2, None, &[]).unwrap();
        cache.store(&checksum2, original, 300).unwrap();
        assert_eq!(cache.size(), 800);

        // Remove 1
        cache.remove(&checksum1).unwrap();
        assert_eq!(cache.size(), 300);

        // Remove 2
        cache.remove(&checksum2).unwrap();
        assert_eq!(cache.size(), 0);
    }
}
