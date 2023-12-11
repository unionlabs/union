//! This module provides a set of primitives that allow implementing an incremental cache on top of
//! Cranelift, making it possible to reuse previous compiled artifacts for functions that have been
//! compiled previously.
//!
//! This set of operation is experimental and can be enabled using the Cargo feature
//! `incremental-cache`.
//!
//! This can bring speedups in different cases: change-code-and-immediately-recompile iterations
//! get faster, modules sharing lots of code can reuse each other's artifacts, etc.
//!
//! The three main primitives are the following:
//! - `compute_cache_key` is used to compute the cache key associated to a `Function`. This is
//! basically the content of the function, modulo a few things the caching system is resilient to.
//! - `serialize_compiled` is used to serialize the result of a compilation, so it can be reused
//! later on by...
//! - `try_finish_recompile`, which reads binary blobs serialized with `serialize_compiled`,
//! re-creating the compilation artifact from those.
//!
//! The `CacheStore` trait and `Context::compile_with_cache` method are provided as
//! high-level, easy-to-use facilities to make use of that cache, and show an example of how to use
//! the above three primitives to form a full incremental caching system.

use core::fmt;

use crate::alloc::string::String;
use crate::alloc::vec::Vec;
use crate::ir::function::{FunctionStencil, VersionMarker};
use crate::ir::Function;
use crate::machinst::{CompiledCode, CompiledCodeStencil};
use crate::result::CompileResult;
use crate::{isa::TargetIsa, timing};
use crate::{trace, CompileError, Context};
use alloc::borrow::{Cow, ToOwned as _};
use alloc::string::ToString as _;

impl Context {
    /// Compile the function, as in `compile`, but tries to reuse compiled artifacts from former
    /// compilations using the provided cache store.
    pub fn compile_with_cache(
        &mut self,
        isa: &dyn TargetIsa,
        cache_store: &mut dyn CacheKvStore,
    ) -> CompileResult<(&CompiledCode, bool)> {
        let cache_key_hash = {
            let _tt = timing::try_incremental_cache();

            let cache_key_hash = compute_cache_key(isa, &mut self.func);

            if let Some(blob) = cache_store.get(&cache_key_hash.0) {
                match try_finish_recompile(&self.func, &blob) {
                    Ok(compiled_code) => {
                        let info = compiled_code.code_info();

                        if isa.flags().enable_incremental_compilation_cache_checks() {
                            let actual_result = self.compile(isa)?;
                            assert_eq!(*actual_result, compiled_code);
                            assert_eq!(actual_result.code_info(), info);
                            // no need to set `compiled_code` here, it's set by `compile()`.
                            return Ok((actual_result, true));
                        }

                        let compiled_code = self.compiled_code.insert(compiled_code);
                        return Ok((compiled_code, true));
                    }
                    Err(err) => {
                        trace!("error when finishing recompilation: {err}");
                    }
                }
            }

            cache_key_hash
        };

        let stencil = self.compile_stencil(isa).map_err(|err| CompileError {
            inner: err,
            func: &self.func,
        })?;

        let stencil = {
            let _tt = timing::store_incremental_cache();
            let (stencil, res) = serialize_compiled(stencil);
            if let Ok(blob) = res {
                cache_store.insert(&cache_key_hash.0, blob);
            }
            stencil
        };

        let compiled_code = self
            .compiled_code
            .insert(stencil.apply_params(&self.func.params));

        Ok((compiled_code, false))
    }
}

/// Backing storage for an incremental compilation cache, when enabled.
pub trait CacheKvStore {
    /// Given a cache key hash, retrieves the associated opaque serialized data.
    fn get(&self, key: &[u8]) -> Option<Cow<[u8]>>;

    /// Given a new cache key and a serialized blob obtained from `serialize_compiled`, stores it
    /// in the cache store.
    fn insert(&mut self, key: &[u8], val: Vec<u8>);
}

/// Hashed `CachedKey`, to use as an identifier when looking up whether a function has already been
/// compiled or not.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct CacheKeyHash([u8; 32]);

impl std::fmt::Display for CacheKeyHash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CacheKeyHash:{:?}", self.0)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedFunc {
    stencil: CompiledCodeStencil,
    version_marker: VersionMarker,
}

/// Key for caching a single function's compilation.
///
/// If two functions get the same `CacheKey`, then we can reuse the compiled artifacts, modulo some
/// fixups.
///
/// Note: the key will be invalidated across different versions of cranelift, as the
/// `FunctionStencil` contains a `VersionMarker` itself.
#[derive(Hash)]
struct CacheKey<'a> {
    stencil: &'a FunctionStencil,
    parameters: CompileParameters,
}

#[derive(Clone, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
struct CompileParameters {
    isa: String,
    triple: String,
    flags: String,
    isa_flags: Vec<String>,
}

impl CompileParameters {
    fn from_isa(isa: &dyn TargetIsa) -> Self {
        Self {
            isa: isa.name().to_owned(),
            triple: isa.triple().to_string(),
            flags: isa.flags().to_string(),
            isa_flags: isa
                .isa_flags()
                .into_iter()
                .map(|v| v.value_string())
                .collect(),
        }
    }
}

impl<'a> CacheKey<'a> {
    /// Creates a new cache store key for a function.
    ///
    /// This is a bit expensive to compute, so it should be cached and reused as much as possible.
    fn new(isa: &dyn TargetIsa, f: &'a mut Function) -> Self {
        // Make sure the blocks and instructions are sequenced the same way as we might
        // have serialized them earlier. This is the symmetric of what's done in
        // `try_load`.
        f.stencil.layout.full_renumber();
        CacheKey {
            stencil: &f.stencil,
            parameters: CompileParameters::from_isa(isa),
        }
    }
}

/// Compute a cache key, and hash it on your behalf.
///
/// Since computing the `CacheKey` is a bit expensive, it should be done as least as possible.
pub fn compute_cache_key(isa: &dyn TargetIsa, func: &mut Function) -> CacheKeyHash {
    use core::hash::{Hash as _, Hasher};
    use sha2::Digest as _;

    struct Sha256Hasher(sha2::Sha256);

    impl Hasher for Sha256Hasher {
        fn finish(&self) -> u64 {
            panic!("Sha256Hasher doesn't support finish!");
        }
        fn write(&mut self, bytes: &[u8]) {
            self.0.update(bytes);
        }
    }

    let cache_key = CacheKey::new(isa, func);

    let mut hasher = Sha256Hasher(sha2::Sha256::new());
    cache_key.hash(&mut hasher);
    let hash: [u8; 32] = hasher.0.finalize().into();

    CacheKeyHash(hash)
}

/// Given a function that's been successfully compiled, serialize it to a blob that the caller may
/// store somewhere for future use by `try_finish_recompile`.
///
/// As this function requires ownership on the `CompiledCodeStencil`, it gives it back at the end
/// of the function call. The value is left untouched.
pub fn serialize_compiled(
    result: CompiledCodeStencil,
) -> (CompiledCodeStencil, Result<Vec<u8>, bincode::Error>) {
    let cached = CachedFunc {
        stencil: result,
        version_marker: VersionMarker,
    };
    let result = bincode::serialize(&cached);
    (cached.stencil, result)
}

/// An error returned when recompiling failed.
#[derive(Debug)]
pub enum RecompileError {
    /// The version embedded in the cache entry isn't the same as cranelift's current version.
    VersionMismatch,
    /// An error occurred while deserializing the cache entry.
    Deserialize(bincode::Error),
}

impl fmt::Display for RecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecompileError::VersionMismatch => write!(f, "cranelift version mismatch",),
            RecompileError::Deserialize(err) => {
                write!(f, "bincode failed during deserialization: {err}")
            }
        }
    }
}

/// Given a function that's been precompiled and its entry in the caching storage, try to shortcut
/// compilation of the given function.
///
/// Precondition: the bytes must have retrieved from a cache store entry which hash value
/// is strictly the same as the `Function`'s computed hash retrieved from `compute_cache_key`.
pub fn try_finish_recompile(func: &Function, bytes: &[u8]) -> Result<CompiledCode, RecompileError> {
    match bincode::deserialize::<CachedFunc>(bytes) {
        Ok(result) => {
            if result.version_marker != func.stencil.version_marker {
                Err(RecompileError::VersionMismatch)
            } else {
                Ok(result.stencil.apply_params(&func.params))
            }
        }
        Err(err) => Err(RecompileError::Deserialize(err)),
    }
}
