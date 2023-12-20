use std::any::Any;

use crate::jsc::vm::VMExternRef;
use crate::store::{AsStoreMut, AsStoreRef};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ExternRef;

impl ExternRef {
    pub fn new<T>(_store: &mut impl AsStoreMut, _value: T) -> Self
    where
        T: Any + Send + Sync + 'static + Sized,
    {
        unimplemented!("ExternRef is not yet supported in Javascript");
    }

    pub fn downcast<'a, T>(&self, _store: &'a impl AsStoreRef) -> Option<&'a T>
    where
        T: Any + Send + Sync + 'static + Sized,
    {
        unimplemented!("ExternRef is not yet supported in Javascript");
    }

    pub(crate) fn vm_externref(&self) -> VMExternRef {
        unimplemented!("ExternRef is not yet supported in Javascript");
    }

    pub(crate) unsafe fn from_vm_externref(
        _store: &mut impl AsStoreMut,
        _vm_externref: VMExternRef,
    ) -> Self {
        unimplemented!("ExternRef is not yet supported in Javascript");
    }

    pub fn is_from_store(&self, _store: &impl AsStoreRef) -> bool {
        true
    }
}
