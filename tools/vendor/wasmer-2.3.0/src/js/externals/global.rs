use crate::js::export::Export;
use crate::js::export::VMGlobal;
use crate::js::exports::{ExportError, Exportable};
use crate::js::externals::Extern;
use crate::js::store::Store;
use crate::js::types::{Val, ValType};
use crate::js::wasm_bindgen_polyfill::Global as JSGlobal;
use crate::js::GlobalType;
use crate::js::Mutability;
use crate::js::RuntimeError;
use wasm_bindgen::JsValue;

/// A WebAssembly `global` instance.
///
/// A global instance is the runtime representation of a global variable.
/// It consists of an individual value and a flag indicating whether it is mutable.
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#global-instances>
#[derive(Debug, Clone, PartialEq)]
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
        let global_ty = GlobalType {
            mutability,
            ty: val.ty(),
        };
        let descriptor = js_sys::Object::new();
        let (type_str, value) = match val {
            Val::I32(i) => ("i32", JsValue::from_f64(i as _)),
            Val::I64(i) => ("i64", JsValue::from_f64(i as _)),
            Val::F32(f) => ("f32", JsValue::from_f64(f as _)),
            Val::F64(f) => ("f64", JsValue::from_f64(f)),
            _ => unimplemented!("The type is not yet supported in the JS Global API"),
        };
        // This is the value type as string, even though is incorrectly called "value"
        // in the JS API.
        js_sys::Reflect::set(&descriptor, &"value".into(), &type_str.into())?;
        js_sys::Reflect::set(
            &descriptor,
            &"mutable".into(),
            &mutability.is_mutable().into(),
        )?;

        let js_global = JSGlobal::new(&descriptor, &value).unwrap();
        let global = VMGlobal::new(js_global, global_ty);

        Ok(Self {
            store: store.clone(),
            vm_global: global,
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
        &self.vm_global.ty
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
        match self.vm_global.ty.ty {
            ValType::I32 => Val::I32(self.vm_global.global.value().as_f64().unwrap() as _),
            ValType::I64 => Val::I64(self.vm_global.global.value().as_f64().unwrap() as _),
            ValType::F32 => Val::F32(self.vm_global.global.value().as_f64().unwrap() as _),
            ValType::F64 => Val::F64(self.vm_global.global.value().as_f64().unwrap()),
            _ => unimplemented!("The type is not yet supported in the JS Global API"),
        }
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
        if self.vm_global.ty.mutability == Mutability::Const {
            return Err(RuntimeError::new("The global is immutable".to_owned()));
        }
        if val.ty() != self.vm_global.ty.ty {
            return Err(RuntimeError::new("The types don't match".to_owned()));
        }
        let new_value = match val {
            Val::I32(i) => JsValue::from_f64(i as _),
            Val::I64(i) => JsValue::from_f64(i as _),
            Val::F32(f) => JsValue::from_f64(f as _),
            Val::F64(f) => JsValue::from_f64(f),
            _ => unimplemented!("The type is not yet supported in the JS Global API"),
        };
        self.vm_global.global.set_value(&new_value);
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
        self.vm_global == other.vm_global
    }
}

impl<'a> Exportable<'a> for Global {
    fn to_export(&self) -> Export {
        Export::Global(self.vm_global.clone())
    }

    fn get_self_from_extern(_extern: &'a Extern) -> Result<&'a Self, ExportError> {
        match _extern {
            Extern::Global(global) => Ok(global),
            _ => Err(ExportError::IncompatibleType),
        }
    }
}
