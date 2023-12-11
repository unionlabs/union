use std::any::Any;

use crate::store::{AsStoreMut, AsStoreRef};

#[cfg(feature = "js")]
use crate::js::extern_ref as extern_ref_imp;
#[cfg(feature = "jsc")]
use crate::jsc::extern_ref as extern_ref_imp;
#[cfg(feature = "sys")]
use crate::sys::extern_ref as extern_ref_imp;
use crate::vm::VMExternRef;

#[derive(Debug, Clone)]
#[repr(transparent)]
/// An opaque reference to some data. This reference can be passed through Wasm.
pub struct ExternRef(pub(crate) extern_ref_imp::ExternRef);

impl ExternRef {
    /// Make a new extern reference
    pub fn new<T>(store: &mut impl AsStoreMut, value: T) -> Self
    where
        T: Any + Send + Sync + 'static + Sized,
    {
        Self(extern_ref_imp::ExternRef::new(store, value))
    }

    /// Try to downcast to the given value.
    pub fn downcast<'a, T>(&self, store: &'a impl AsStoreRef) -> Option<&'a T>
    where
        T: Any + Send + Sync + 'static + Sized,
    {
        self.0.downcast(store)
    }

    pub(crate) fn vm_externref(&self) -> VMExternRef {
        self.0.vm_externref()
    }

    pub(crate) unsafe fn from_vm_externref(
        store: &mut impl AsStoreMut,
        vm_externref: VMExternRef,
    ) -> Self {
        Self(extern_ref_imp::ExternRef::from_vm_externref(
            store,
            vm_externref,
        ))
    }

    /// Checks whether this `ExternRef` can be used with the given context.
    ///
    /// Primitive (`i32`, `i64`, etc) and null funcref/externref values are not
    /// tied to a context and can be freely shared between contexts.
    ///
    /// Externref and funcref values are tied to a context and can only be used
    /// with that context.
    pub fn is_from_store(&self, store: &impl AsStoreRef) -> bool {
        self.0.is_from_store(store)
    }
}
