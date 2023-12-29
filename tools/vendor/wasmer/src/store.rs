use crate::engine::{AsEngineRef, Engine, EngineRef};
use derivative::Derivative;
use std::{
    fmt,
    ops::{Deref, DerefMut},
};
#[cfg(feature = "sys")]
pub use wasmer_compiler::Tunables;
pub use wasmer_types::{OnCalledAction, StoreId};
#[cfg(feature = "sys")]
use wasmer_vm::init_traps;
#[cfg(feature = "sys")]
pub use wasmer_vm::TrapHandlerFn;

#[cfg(feature = "sys")]
pub use wasmer_vm::{StoreHandle, StoreObjects};

#[cfg(feature = "js")]
pub use crate::js::store::{StoreHandle, StoreObjects};

#[cfg(feature = "jsc")]
pub use crate::jsc::store::{StoreHandle, StoreObjects};

/// Call handler for a store.
// TODO: better documentation!
pub type OnCalledHandler = Box<
    dyn FnOnce(StoreMut<'_>) -> Result<OnCalledAction, Box<dyn std::error::Error + Send + Sync>>,
>;

/// We require the context to have a fixed memory address for its lifetime since
/// various bits of the VM have raw pointers that point back to it. Hence we
/// wrap the actual context in a box.
#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct StoreInner {
    pub(crate) objects: StoreObjects,
    #[derivative(Debug = "ignore")]
    pub(crate) engine: Engine,
    #[cfg(feature = "sys")]
    #[derivative(Debug = "ignore")]
    pub(crate) trap_handler: Option<Box<TrapHandlerFn<'static>>>,
    #[derivative(Debug = "ignore")]
    pub(crate) on_called: Option<OnCalledHandler>,
}

/// The store represents all global state that can be manipulated by
/// WebAssembly programs. It consists of the runtime representation
/// of all instances of functions, tables, memories, and globals that
/// have been allocated during the lifetime of the abstract machine.
///
/// The `Store` holds the engine (that is —amongst many things— used to compile
/// the Wasm bytes into a valid module artifact).
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#store>
pub struct Store {
    pub(crate) inner: Box<StoreInner>,
}

impl Store {
    /// Creates a new `Store` with a specific [`Engine`].
    pub fn new(engine: impl Into<Engine>) -> Self {
        // Make sure the signal handlers are installed.
        // This is required for handling traps.
        #[cfg(feature = "sys")]
        init_traps();

        Self {
            inner: Box::new(StoreInner {
                objects: Default::default(),
                engine: engine.into(),
                #[cfg(feature = "sys")]
                trap_handler: None,
                on_called: None,
            }),
        }
    }

    #[cfg(feature = "sys")]
    /// Set the trap handler in this store.
    pub fn set_trap_handler(&mut self, handler: Option<Box<TrapHandlerFn<'static>>>) {
        self.inner.trap_handler = handler;
    }

    /// Returns the [`Engine`].
    pub fn engine(&self) -> &Engine {
        &self.inner.engine
    }

    /// Checks whether two stores are identical. A store is considered
    /// equal to another store if both have the same engine.
    pub fn same(a: &Self, b: &Self) -> bool {
        a.id() == b.id()
    }

    /// Returns the ID of this store
    pub fn id(&self) -> StoreId {
        self.inner.objects.id()
    }
}

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        Self::same(self, other)
    }
}

// This is required to be able to set the trap_handler in the
// Store.
unsafe impl Send for Store {}
unsafe impl Sync for Store {}

impl Default for Store {
    fn default() -> Self {
        Self::new(Engine::default())
    }
}

impl AsStoreRef for Store {
    fn as_store_ref(&self) -> StoreRef<'_> {
        StoreRef { inner: &self.inner }
    }
}
impl AsStoreMut for Store {
    fn as_store_mut(&mut self) -> StoreMut<'_> {
        StoreMut {
            inner: &mut self.inner,
        }
    }
    fn objects_mut(&mut self) -> &mut StoreObjects {
        &mut self.inner.objects
    }
}

impl AsEngineRef for Store {
    fn as_engine_ref(&self) -> EngineRef<'_> {
        EngineRef::new(&self.inner.engine)
    }
}

impl AsEngineRef for StoreRef<'_> {
    fn as_engine_ref(&self) -> EngineRef<'_> {
        EngineRef::new(&self.inner.engine)
    }
}

impl AsEngineRef for StoreMut<'_> {
    fn as_engine_ref(&self) -> EngineRef<'_> {
        EngineRef::new(&self.inner.engine)
    }
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Store").finish()
    }
}

/// A temporary handle to a [`Store`].
#[derive(Debug)]
pub struct StoreRef<'a> {
    pub(crate) inner: &'a StoreInner,
}

impl<'a> StoreRef<'a> {
    pub(crate) fn objects(&self) -> &'a StoreObjects {
        &self.inner.objects
    }

    /// Returns the [`Engine`].
    pub fn engine(&self) -> &Engine {
        &self.inner.engine
    }

    /// Checks whether two stores are identical. A store is considered
    /// equal to another store if both have the same engine.
    pub fn same(a: &Self, b: &Self) -> bool {
        a.inner.objects.id() == b.inner.objects.id()
    }

    /// The signal handler
    #[cfg(feature = "sys")]
    #[inline]
    pub fn signal_handler(&self) -> Option<*const TrapHandlerFn<'static>> {
        self.inner
            .trap_handler
            .as_ref()
            .map(|handler| handler.as_ref() as *const _)
    }
}

/// A temporary handle to a [`Store`].
pub struct StoreMut<'a> {
    pub(crate) inner: &'a mut StoreInner,
}

impl<'a> StoreMut<'a> {
    /// Returns the [`Engine`].
    pub fn engine(&self) -> &Engine {
        &self.inner.engine
    }

    /// Checks whether two stores are identical. A store is considered
    /// equal to another store if both have the same engine.
    pub fn same(a: &Self, b: &Self) -> bool {
        a.inner.objects.id() == b.inner.objects.id()
    }

    #[allow(unused)]
    pub(crate) fn engine_and_objects_mut(&mut self) -> (&Engine, &mut StoreObjects) {
        (&self.inner.engine, &mut self.inner.objects)
    }

    pub(crate) fn as_raw(&self) -> *mut StoreInner {
        self.inner as *const StoreInner as *mut StoreInner
    }

    pub(crate) unsafe fn from_raw(raw: *mut StoreInner) -> Self {
        Self { inner: &mut *raw }
    }

    // TODO: OnCalledAction is needed for asyncify. It will be refactored with https://github.com/wasmerio/wasmer/issues/3451
    /// Sets the unwind callback which will be invoked when the call finishes
    pub fn on_called<F>(&mut self, callback: F)
    where
        F: FnOnce(StoreMut<'_>) -> Result<OnCalledAction, Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        self.inner.on_called.replace(Box::new(callback));
    }
}

/// Helper trait for a value that is convertible to a [`StoreRef`].
pub trait AsStoreRef {
    /// Returns a `StoreRef` pointing to the underlying context.
    fn as_store_ref(&self) -> StoreRef<'_>;
}

/// Helper trait for a value that is convertible to a [`StoreMut`].
pub trait AsStoreMut: AsStoreRef {
    /// Returns a `StoreMut` pointing to the underlying context.
    fn as_store_mut(&mut self) -> StoreMut<'_>;

    /// Returns the ObjectMutable
    fn objects_mut(&mut self) -> &mut StoreObjects;
}

impl AsStoreRef for StoreRef<'_> {
    fn as_store_ref(&self) -> StoreRef<'_> {
        StoreRef { inner: self.inner }
    }
}

impl AsStoreRef for StoreMut<'_> {
    fn as_store_ref(&self) -> StoreRef<'_> {
        StoreRef { inner: self.inner }
    }
}
impl AsStoreMut for StoreMut<'_> {
    fn as_store_mut(&mut self) -> StoreMut<'_> {
        StoreMut { inner: self.inner }
    }
    fn objects_mut(&mut self) -> &mut StoreObjects {
        &mut self.inner.objects
    }
}

impl<P> AsStoreRef for P
where
    P: Deref,
    P::Target: AsStoreRef,
{
    fn as_store_ref(&self) -> StoreRef<'_> {
        (**self).as_store_ref()
    }
}

impl<P> AsStoreMut for P
where
    P: DerefMut,
    P::Target: AsStoreMut,
{
    fn as_store_mut(&mut self) -> StoreMut<'_> {
        (**self).as_store_mut()
    }

    fn objects_mut(&mut self) -> &mut StoreObjects {
        (**self).objects_mut()
    }
}
