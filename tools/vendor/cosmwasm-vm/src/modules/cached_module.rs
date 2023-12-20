use wasmer::Module;

#[derive(Debug, Clone)]
pub struct CachedModule {
    pub module: Module,
    /// The estimated size of this element in memory.
    /// Since the cached modules are just [rkyv](https://rkyv.org/) dumps of the Module
    /// instances, we use the file size of the module on disk (not the Wasm!)
    /// as an estimate for this.
    /// Note: Since CosmWasm 1.4 (Wasmer 4), Store/Engine are not cached anymore.
    /// The majority of the Module size is the Artifact.
    pub size_estimate: usize,
}
