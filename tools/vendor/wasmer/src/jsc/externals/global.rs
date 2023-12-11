use crate::errors::RuntimeError;
use crate::jsc::as_js::param_from_js;
use crate::jsc::as_js::AsJs;
use crate::store::{AsStoreMut, AsStoreRef};
use crate::value::Value;
use crate::vm::{VMExtern, VMGlobal};
use crate::GlobalType;
use crate::Mutability;
use rusty_jsc::{JSObject, JSValue};
use wasmer_types::{RawValue, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Global {
    pub(crate) handle: VMGlobal,
}

// Global can't be Send in js because it dosen't support `structuredClone`
// https://developer.mozilla.org/en-US/docs/Web/API/structuredClone
// unsafe impl Send for Global {}

impl Global {
    pub(crate) fn to_vm_extern(&self) -> VMExtern {
        VMExtern::Global(self.handle.clone())
    }

    /// Create a `Global` with the initial value [`Value`] and the provided [`Mutability`].
    pub(crate) fn from_value(
        store: &mut impl AsStoreMut,
        val: Value,
        mutability: Mutability,
    ) -> Result<Self, RuntimeError> {
        if !val.is_from_store(store) {
            return Err(RuntimeError::new("cross-`Store` values are not supported"));
        }
        let global_ty = GlobalType {
            mutability,
            ty: val.ty(),
        };
        let store_mut = store.as_store_mut();
        let engine = store_mut.engine();
        let context = engine.0.context();

        let mut descriptor = JSObject::new(&context);
        let type_str = match val.ty() {
            Type::I32 => "i32",
            Type::I64 => "i64",
            Type::F32 => "f32",
            Type::F64 => "f64",
            ty => unimplemented!(
                "The type: {:?} is not yet supported in the JS Global API",
                ty
            ),
        };
        // This is the value type as string, even though is incorrectly called "value"
        // in the JS API.
        descriptor.set_property(
            &context,
            "value".to_string(),
            JSValue::string(&context, type_str.to_string()),
        );
        descriptor.set_property(
            &context,
            "mutable".to_string(),
            JSValue::boolean(&context, mutability.is_mutable()),
        );

        let value: JSValue = val.as_jsvalue(&store_mut);
        let js_global = engine
            .0
            .wasm_global_type()
            .construct(&context, &[descriptor.to_jsvalue(), value])
            .map_err(|e| <JSValue as Into<RuntimeError>>::into(e))?;
        let vm_global = VMGlobal::new(js_global, global_ty);
        Ok(Self::from_vm_extern(store, vm_global))
    }

    pub fn ty(&self, _store: &impl AsStoreRef) -> GlobalType {
        self.handle.ty
    }

    pub fn get(&self, store: &mut impl AsStoreMut) -> Value {
        let store_mut = store.as_store_mut();
        let engine = store_mut.engine();
        let context = engine.0.context();
        let value = self
            .handle
            .global
            .get_property(&context, "value".to_string());
        param_from_js(&context, &self.handle.ty.ty, &value)
    }

    pub fn set(&self, store: &mut impl AsStoreMut, val: Value) -> Result<(), RuntimeError> {
        let store_mut = store.as_store_mut();
        let new_value = val.as_jsvalue(&store_mut);
        let engine = store_mut.engine();
        let context = engine.0.context();
        self.handle
            .global
            .set_property(&context, "value".to_string(), new_value)
            .map_err(|e| <JSValue as Into<RuntimeError>>::into(e))
    }

    pub(crate) fn from_vm_extern(store: &mut impl AsStoreMut, vm_global: VMGlobal) -> Self {
        use crate::jsc::store::StoreObject;
        VMGlobal::list_mut(store.objects_mut()).push(vm_global.clone());
        Self { handle: vm_global }
    }

    pub fn is_from_store(&self, _store: &impl AsStoreRef) -> bool {
        true
    }
}
