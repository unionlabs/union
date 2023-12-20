use core::ops::Deref;

#[cfg(feature = "sys")]
use crate::sys::engine as engine_imp;
#[cfg(feature = "sys")]
pub(crate) use crate::sys::engine::default_engine;
#[cfg(feature = "sys")]
use crate::IntoBytes;
#[cfg(feature = "sys")]
use shared_buffer::OwnedBuffer;
#[cfg(feature = "sys")]
use std::path::Path;
#[cfg(feature = "sys")]
use std::sync::Arc;
#[cfg(feature = "sys")]
pub use wasmer_compiler::{Artifact, CompilerConfig, EngineInner, Features, Tunables};
#[cfg(feature = "sys")]
use wasmer_types::DeserializeError;

#[cfg(feature = "js")]
use crate::js::engine as engine_imp;
#[cfg(feature = "js")]
pub(crate) use crate::js::engine::default_engine;

#[cfg(feature = "jsc")]
use crate::jsc::engine as engine_imp;
#[cfg(feature = "jsc")]
pub(crate) use crate::jsc::engine::default_engine;

/// The engine type
#[derive(Clone, Debug)]
pub struct Engine(pub(crate) engine_imp::Engine);

impl Engine {
    /// Returns the deterministic id of this engine
    pub fn deterministic_id(&self) -> &str {
        self.0.deterministic_id()
    }

    #[cfg(all(feature = "sys", not(target_arch = "wasm32")))]
    /// Deserializes a WebAssembly module which was previously serialized with
    /// `Module::serialize`.
    ///
    /// NOTE: you should almost always prefer [`Self::deserialize`].
    ///
    /// # Safety
    /// See [`Artifact::deserialize_unchecked`].
    pub unsafe fn deserialize_unchecked(
        &self,
        bytes: impl IntoBytes,
    ) -> Result<Arc<Artifact>, DeserializeError> {
        Ok(Arc::new(Artifact::deserialize_unchecked(
            &self.0,
            bytes.into_bytes().into(),
        )?))
    }

    #[cfg(all(feature = "sys", not(target_arch = "wasm32")))]
    /// Deserializes a WebAssembly module which was previously serialized with
    /// `Module::serialize`.
    ///
    /// # Safety
    /// See [`Artifact::deserialize`].
    pub unsafe fn deserialize(
        &self,
        bytes: impl IntoBytes,
    ) -> Result<Arc<Artifact>, DeserializeError> {
        Ok(Arc::new(Artifact::deserialize(
            &self.0,
            bytes.into_bytes().into(),
        )?))
    }

    #[cfg(all(feature = "sys", not(target_arch = "wasm32")))]
    /// Load a serialized WebAssembly module from a file and deserialize it.
    ///
    /// NOTE: you should almost always prefer [`Self::deserialize_from_file`].
    ///
    /// # Safety
    /// See [`Artifact::deserialize_unchecked`].
    pub unsafe fn deserialize_from_file_unchecked(
        &self,
        file_ref: &Path,
    ) -> Result<Arc<Artifact>, DeserializeError> {
        let file = std::fs::File::open(file_ref)?;
        Ok(Arc::new(Artifact::deserialize_unchecked(
            &self.0,
            OwnedBuffer::from_file(&file)
                .map_err(|e| DeserializeError::Generic(format!("{e:?}")))?,
        )?))
    }

    #[cfg(all(feature = "sys", not(target_arch = "wasm32")))]
    /// Load a serialized WebAssembly module from a file and deserialize it.
    ///
    /// # Safety
    /// See [`Artifact::deserialize`].
    pub unsafe fn deserialize_from_file(
        &self,
        file_ref: &Path,
    ) -> Result<Arc<Artifact>, DeserializeError> {
        let bytes = std::fs::read(file_ref)?;
        Ok(Arc::new(Artifact::deserialize(&self.0, bytes.into())?))
    }
}

impl AsEngineRef for Engine {
    #[inline]
    fn as_engine_ref(&self) -> EngineRef {
        EngineRef { inner: self }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self(default_engine())
    }
}

impl<T: Into<engine_imp::Engine>> From<T> for Engine {
    fn from(t: T) -> Self {
        Self(t.into())
    }
}

/// A temporary handle to an [`Engine`]
/// EngineRef can be used to build a [`Module`][super::Module]
/// It can be created directly with an [`Engine`]
/// Or from anything implementing [`AsEngineRef`]
/// like from [`Store`][super::Store] typicaly.
pub struct EngineRef<'a> {
    /// The inner engine
    pub(crate) inner: &'a Engine,
}

impl<'a> EngineRef<'a> {
    /// Get inner [`Engine`]
    pub fn engine(&self) -> &Engine {
        self.inner
    }
    /// Create an EngineRef from an Engine
    pub fn new(engine: &'a Engine) -> Self {
        EngineRef { inner: engine }
    }
}

/// Helper trait for a value that is convertible to a [`EngineRef`].
pub trait AsEngineRef {
    /// Returns a `EngineRef` pointing to the underlying context.
    fn as_engine_ref(&self) -> EngineRef<'_>;
}

impl AsEngineRef for EngineRef<'_> {
    #[inline]
    fn as_engine_ref(&self) -> EngineRef<'_> {
        EngineRef { inner: self.inner }
    }
}

impl<P> AsEngineRef for P
where
    P: Deref,
    P::Target: AsEngineRef,
{
    #[inline]
    fn as_engine_ref(&self) -> EngineRef<'_> {
        (**self).as_engine_ref()
    }
}
