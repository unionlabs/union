use crate::imports::Imports;
use crate::instance::Instance;
use crate::jsc::engine::JSC;
use crate::jsc::instance::Instance as JsInstance;
use crate::jsc::vm::{VMFunction, VMGlobal, VMMemory, VMTable};
use crate::store::{AsStoreMut, AsStoreRef};
use crate::value::Value;
use crate::Type;
use crate::{Extern, Function, Global, Memory, Table};
use rusty_jsc::{JSContext, JSObject, JSValue};
use std::collections::HashMap;
use std::convert::TryInto;
use wasmer_types::ExternType;

/// Convert the given type to a [`JsValue`].
pub trait AsJs: Sized {
    /// The inner definition type from this Javascript object
    type DefinitionType;
    /// Convert the given type to a [`JsValue`].
    fn as_jsvalue(&self, store: &impl AsStoreRef) -> JSValue;
    /// Convert the given type to a [`JsValue`].
    fn from_jsvalue(
        store: &mut impl AsStoreMut,
        type_: &Self::DefinitionType,
        value: &JSValue,
    ) -> Result<Self, JSValue>;
}

#[inline]
pub fn param_from_js(context: &JSContext, ty: &Type, js_val: &JSValue) -> Value {
    match ty {
        Type::I32 => Value::I32(js_val.to_number(&context).unwrap() as _),
        Type::I64 => {
            let number = if js_val.is_number(&context) {
                js_val.to_number(&context).unwrap() as _
            } else {
                js_val
                    .to_string(&context)
                    .unwrap()
                    .to_string()
                    .parse()
                    .unwrap()
            };
            Value::I64(number)
        }
        Type::F32 => Value::F32(js_val.to_number(&context).unwrap() as _),
        Type::F64 => Value::F64(js_val.to_number(&context).unwrap()),
        Type::V128 => {
            let number = if js_val.is_number(&context) {
                js_val.to_number(&context).unwrap() as _
            } else {
                js_val
                    .to_string(&context)
                    .unwrap()
                    .to_string()
                    .parse()
                    .unwrap()
            };
            Value::V128(number)
        }
        Type::ExternRef | Type::FuncRef => unimplemented!(
            "The type `{:?}` is not yet supported in the JS Function API",
            ty
        ),
    }
}

impl AsJs for Value {
    type DefinitionType = Type;

    fn as_jsvalue(&self, store: &impl AsStoreRef) -> JSValue {
        let engine = store.as_store_ref();
        let context = engine.jsc().context();
        match self {
            Self::I32(i) => JSValue::number(&context, *i as _),
            // JavascriptCore will fail with:
            // new WebAssembly.Global({value: "i64", mutable: false}, 3);
            // But will succeed with
            // new WebAssembly.Global({value: "i64", mutable: false}, "3");
            Self::I64(i) => JSValue::string(&context, (*i).to_string()),
            Self::F32(f) => JSValue::number(&context, *f as _),
            Self::F64(f) => JSValue::number(&context, *f),
            Self::V128(v) => JSValue::number(&context, *v as _),
            Self::FuncRef(Some(func)) => func.0.handle.function.clone().to_jsvalue(),
            Self::FuncRef(None) => JSValue::null(&context),
            Self::ExternRef(_) => unimplemented!(),
        }
    }

    fn from_jsvalue(
        store: &mut impl AsStoreMut,
        type_: &Self::DefinitionType,
        value: &JSValue,
    ) -> Result<Self, JSValue> {
        let engine = store.as_store_ref();
        let context = engine.jsc().context();
        Ok(param_from_js(context, type_, value))
    }
}

impl AsJs for Extern {
    type DefinitionType = ExternType;

    fn as_jsvalue(&self, _store: &impl AsStoreRef) -> JSValue {
        match self {
            Self::Memory(memory) => memory.0.handle.memory.clone().to_jsvalue(),
            Self::Function(function) => function.0.handle.function.clone().to_jsvalue(),
            Self::Table(table) => table.0.handle.table.clone().to_jsvalue(),
            Self::Global(global) => global.0.handle.global.clone().to_jsvalue(),
        }
    }

    fn from_jsvalue(
        store: &mut impl AsStoreMut,
        extern_type: &Self::DefinitionType,
        val: &JSValue,
    ) -> Result<Self, JSValue> {
        // Note: this function do a soft check over the type
        // We only check the "kind" of Extern, but nothing else
        // unimplemented!();
        let engine = store.as_store_mut();
        let context = engine.jsc().context();
        match extern_type {
            ExternType::Function(function_type) => {
                let obj_val = val.to_object(&context).unwrap();
                Ok(Self::Function(Function::from_vm_extern(
                    store,
                    VMFunction::new(obj_val, function_type.clone()),
                )))
            }
            ExternType::Global(global_type) => {
                let obj_val = val.to_object(&context).unwrap();
                Ok(Self::Global(Global::from_vm_extern(
                    store,
                    VMGlobal::new(obj_val, global_type.clone()),
                )))
            }
            ExternType::Memory(memory_type) => {
                let obj_val = val.to_object(&context).unwrap();
                Ok(Self::Memory(Memory::from_vm_extern(
                    store,
                    VMMemory::new(obj_val, memory_type.clone()),
                )))
            }
            ExternType::Table(table_type) => {
                let obj_val = val.to_object(&context).unwrap();
                Ok(Self::Table(Table::from_vm_extern(
                    store,
                    VMTable::new(obj_val, table_type.clone()),
                )))
            }
        }
    }
}
