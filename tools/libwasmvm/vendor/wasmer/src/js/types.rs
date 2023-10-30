use crate::js::externals::Function;
// use crate::js::store::{Store, StoreObject};
// use crate::js::RuntimeError;
use wasm_bindgen::JsValue;
use wasmer_types::Value;
pub use wasmer_types::{
    ExportType, ExternType, FunctionType, GlobalType, ImportType, MemoryType, Mutability,
    TableType, Type as ValType,
};

/// WebAssembly computations manipulate values of basic value types:
/// * Integers (32 or 64 bit width)
/// * Floating-point (32 or 64 bit width)
/// * Vectors (128 bits, with 32 or 64 bit lanes)
///
/// Spec: <https://webassembly.github.io/spec/core/exec/runtime.html#values>
// pub type Val = ();
pub type Val = Value<Function>;

pub trait AsJs {
    fn as_jsvalue(&self) -> JsValue;
}

#[inline]
pub fn param_from_js(ty: &ValType, js_val: &JsValue) -> Val {
    match ty {
        ValType::I32 => Val::I32(js_val.as_f64().unwrap() as _),
        ValType::I64 => Val::I64(js_val.as_f64().unwrap() as _),
        ValType::F32 => Val::F32(js_val.as_f64().unwrap() as _),
        ValType::F64 => Val::F64(js_val.as_f64().unwrap()),
        t => unimplemented!(
            "The type `{:?}` is not yet supported in the JS Function API",
            t
        ),
    }
}

impl AsJs for Val {
    fn as_jsvalue(&self) -> JsValue {
        match self {
            Self::I32(i) => JsValue::from_f64(*i as f64),
            Self::I64(i) => JsValue::from_f64(*i as f64),
            Self::F32(f) => JsValue::from_f64(*f as f64),
            Self::F64(f) => JsValue::from_f64(*f),
            Self::FuncRef(func) => func.as_ref().unwrap().exported.function.clone().into(),
            v => unimplemented!(
                "The value `{:?}` is not yet supported in the JS Function API",
                v
            ),
        }
    }
}
