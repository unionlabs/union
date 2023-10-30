use crate::sys::exports::{ExportError, Exportable};
use crate::sys::externals::Extern;
use crate::sys::store::{Store, StoreObject};
use crate::sys::types::Val;
use crate::sys::GlobalType;
use crate::sys::Mutability;
use crate::sys::RuntimeError;
use loupe::MemoryUsage;
use std::fmt;
use std::sync::Arc;
use wasmer_engine::Export;
use wasmer_vm::{Global as RuntimeGlobal, VMGlobal};

/// A WebAssembly `global` instance.
///
/// A global instance is the runtime representation of a global variable.
/// It consists of an individual value and a flag indicating whether it is mutable.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#global-instances>
#[derive(MemoryUsage)]
pub struct Global {
    store: Store,
    vm_global: VMGlobal,
}

impl Global {
    /// Create a new `Global` with the initial value [`Val`].
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Global, Mutability, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let g = Global::new(&store, Value::I32(1));
    ///
    /// assert_eq!(g.get(), Value::I32(1));
    /// assert_eq!(g.ty().mutability, Mutability::Const);
    /// ```
    pub fn new(store: &Store, val: Val) -> Self {
        Self::from_value(store, val, Mutability::Const).unwrap()
    }

    /// Create a mutable `Global` with the initial value [`Val`].
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Global, Mutability, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let g = Global::new_mut(&store, Value::I32(1));
    ///
    /// assert_eq!(g.get(), Value::I32(1));
    /// assert_eq!(g.ty().mutability, Mutability::Var);
    /// ```
    pub fn new_mut(store: &Store, val: Val) -> Self {
        Self::from_value(store, val, Mutability::Var).unwrap()
    }

    /// Create a `Global` with the initial value [`Val`] and the provided [`Mutability`].
    fn from_value(store: &Store, val: Val, mutability: Mutability) -> Result<Self, RuntimeError> {
        if !val.comes_from_same_store(store) {
            return Err(RuntimeError::new("cross-`Store` globals are not supported"));
        }
        let global = RuntimeGlobal::new(GlobalType {
            mutability,
            ty: val.ty(),
        });
        unsafe {
            global
                .set_unchecked(val.clone())
                .map_err(|e| RuntimeError::new(format!("create global for {:?}: {}", val, e)))?;
        };

        Ok(Self {
            store: store.clone(),
            vm_global: VMGlobal {
                from: Arc::new(global),
                instance_ref: None,
            },
        })
    }

    /// Returns the [`GlobalType`] of the `Global`.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Global, Mutability, Store, Type, Value, GlobalType};
    /// # let store = Store::default();
    /// #
    /// let c = Global::new(&store, Value::I32(1));
    /// let v = Global::new_mut(&store, Value::I64(1));
    ///
    /// assert_eq!(c.ty(), &GlobalType::new(Type::I32, Mutability::Const));
    /// assert_eq!(v.ty(), &GlobalType::new(Type::I64, Mutability::Var));
    /// ```
    pub fn ty(&self) -> &GlobalType {
        self.vm_global.from.ty()
    }

    /// Returns the [`Store`] where the `Global` belongs.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Global, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let g = Global::new(&store, Value::I32(1));
    ///
    /// assert_eq!(g.store(), &store);
    /// ```
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Retrieves the current value [`Val`] that the Global has.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Global, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let g = Global::new(&store, Value::I32(1));
    ///
    /// assert_eq!(g.get(), Value::I32(1));
    /// ```
    pub fn get(&self) -> Val {
        self.vm_global.from.get(&self.store)
    }

    /// Sets a custom value [`Val`] to the runtime Global.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Global, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let g = Global::new_mut(&store, Value::I32(1));
    ///
    /// assert_eq!(g.get(), Value::I32(1));
    ///
    /// g.set(Value::I32(2));
    ///
    /// assert_eq!(g.get(), Value::I32(2));
    /// ```
    ///
    /// # Errors
    ///
    /// Trying to mutate a immutable global will raise an error:
    ///
    /// ```should_panic
    /// # use wasmer::{Global, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let g = Global::new(&store, Value::I32(1));
    ///
    /// g.set(Value::I32(2)).unwrap();
    /// ```
    ///
    /// Trying to set a value of a incompatible type will raise an error:
    ///
    /// ```should_panic
    /// # use wasmer::{Global, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let g = Global::new(&store, Value::I32(1));
    ///
    /// // This results in an error: `RuntimeError`.
    /// g.set(Value::I64(2)).unwrap();
    /// ```
    pub fn set(&self, val: Val) -> Result<(), RuntimeError> {
        if !val.comes_from_same_store(&self.store) {
            return Err(RuntimeError::new("cross-`Store` values are not supported"));
        }
        unsafe {
            self.vm_global
                .from
                .set(val)
                .map_err(|e| RuntimeError::new(format!("{}", e)))?;
        }
        Ok(())
    }

    pub(crate) fn from_vm_export(store: &Store, vm_global: VMGlobal) -> Self {
        Self {
            store: store.clone(),
            vm_global,
        }
    }

    /// Returns whether or not these two globals refer to the same data.
    ///
    /// # Example
    ///
    /// ```
    /// # use wasmer::{Global, Store, Value};
    /// # let store = Store::default();
    /// #
    /// let g = Global::new(&store, Value::I32(1));
    ///
    /// assert!(g.same(&g));
    /// ```
    pub fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.vm_global.from, &other.vm_global.from)
    }

    /// Get access to the backing VM value for this extern. This function is for
    /// tests it should not be called by users of the Wasmer API.
    ///
    /// # Safety
    /// This function is unsafe to call outside of tests for the wasmer crate
    /// because there is no stability guarantee for the returned type and we may
    /// make breaking changes to it at any time or remove this method.
    #[doc(hidden)]
    pub unsafe fn get_vm_global(&self) -> &VMGlobal {
        &self.vm_global
    }
}

impl Clone for Global {
    fn clone(&self) -> Self {
        let mut vm_global = self.vm_global.clone();
        vm_global.upgrade_instance_ref().unwrap();

        Self {
            store: self.store.clone(),
            vm_global,
        }
    }
}

impl fmt::Debug for Global {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Global")
            .field("ty", &self.ty())
            .field("value", &self.get())
            .finish()
    }
}

impl<'a> Exportable<'a> for Global {
    fn to_export(&self) -> Export {
        self.vm_global.clone().into()
    }

    fn get_self_from_extern(_extern: &'a Extern) -> Result<&'a Self, ExportError> {
        match _extern {
            Extern::Global(global) => Ok(global),
            _ => Err(ExportError::IncompatibleType),
        }
    }

    fn into_weak_instance_ref(&mut self) {
        self.vm_global
            .instance_ref
            .as_mut()
            .map(|v| *v = v.downgrade());
    }
}
