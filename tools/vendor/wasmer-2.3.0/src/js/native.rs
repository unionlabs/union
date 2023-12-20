//! Native Functions.
//!
//! This module creates the helper `NativeFunc` that let us call WebAssembly
//! functions with the native ABI, that is:
//!
//! ```ignore
//! let add_one = instance.exports.get_function("function_name")?;
//! let add_one_native: NativeFunc<i32, i32> = add_one.native().unwrap();
//! ```
use std::marker::PhantomData;

use crate::js::{FromToNativeWasmType, Function, RuntimeError, Store, WasmTypeList};
// use std::panic::{catch_unwind, AssertUnwindSafe};
use crate::js::export::VMFunction;
use crate::js::types::param_from_js;
use js_sys::Array;
use std::iter::FromIterator;
use wasm_bindgen::JsValue;
use wasmer_types::NativeWasmType;

/// A WebAssembly function that can be called natively
/// (using the Native ABI).
#[derive(Clone)]
pub struct NativeFunc<Args = (), Rets = ()> {
    store: Store,
    exported: VMFunction,
    _phantom: PhantomData<(Args, Rets)>,
}

unsafe impl<Args, Rets> Send for NativeFunc<Args, Rets> {}

impl<Args, Rets> NativeFunc<Args, Rets>
where
    Args: WasmTypeList,
    Rets: WasmTypeList,
{
    pub(crate) fn new(store: Store, exported: VMFunction) -> Self {
        Self {
            store,
            exported,
            _phantom: PhantomData,
        }
    }
}

impl<Args, Rets> From<&NativeFunc<Args, Rets>> for VMFunction
where
    Args: WasmTypeList,
    Rets: WasmTypeList,
{
    fn from(other: &NativeFunc<Args, Rets>) -> Self {
        other.exported.clone()
    }
}

impl<Args, Rets> From<NativeFunc<Args, Rets>> for Function
where
    Args: WasmTypeList,
    Rets: WasmTypeList,
{
    fn from(other: NativeFunc<Args, Rets>) -> Self {
        Self {
            store: other.store,
            exported: other.exported,
        }
    }
}

macro_rules! impl_native_traits {
    (  $( $x:ident ),* ) => {
        #[allow(unused_parens, non_snake_case)]
        impl<$( $x , )* Rets> NativeFunc<( $( $x ),* ), Rets>
        where
            $( $x: FromToNativeWasmType, )*
            Rets: WasmTypeList,
        {
            /// Call the typed func and return results.
            pub fn call(&self, $( $x: $x, )* ) -> Result<Rets, RuntimeError> {
                let params_list: Vec<JsValue> = vec![ $( JsValue::from_f64($x.to_native().to_binary() as f64) ),* ];
                let results = self.exported.function.apply(
                    &JsValue::UNDEFINED,
                    &Array::from_iter(params_list.iter())
                )?;
                let mut rets_list_array = Rets::empty_array();
                let mut_rets = rets_list_array.as_mut() as *mut [i128] as *mut i128;
                match Rets::size() {
                    0 => {},
                    1 => unsafe {
                        let ty = Rets::wasm_types()[0];
                        let val = param_from_js(&ty, &results);
                        val.write_value_to(mut_rets);
                    }
                    _n => {
                        let results: Array = results.into();
                        for (i, ret_type) in Rets::wasm_types().iter().enumerate() {
                            let ret = results.get(i as u32);
                            unsafe {
                                let val = param_from_js(&ret_type, &ret);
                                val.write_value_to(mut_rets.add(i));
                            }
                        }
                    }
                }
                Ok(Rets::from_array(rets_list_array))
            }

        }

        #[allow(unused_parens)]
        impl<'a, $( $x, )* Rets> crate::js::exports::ExportableWithGenerics<'a, ($( $x ),*), Rets> for NativeFunc<( $( $x ),* ), Rets>
        where
            $( $x: FromToNativeWasmType, )*
            Rets: WasmTypeList,
        {
            fn get_self_from_extern_with_generics(_extern: &crate::js::externals::Extern) -> Result<Self, crate::js::exports::ExportError> {
                use crate::js::exports::Exportable;
                crate::js::Function::get_self_from_extern(_extern)?.native().map_err(|_| crate::js::exports::ExportError::IncompatibleType)
            }
        }
    };
}

impl_native_traits!();
impl_native_traits!(A1);
impl_native_traits!(A1, A2);
impl_native_traits!(A1, A2, A3);
impl_native_traits!(A1, A2, A3, A4);
impl_native_traits!(A1, A2, A3, A4, A5);
impl_native_traits!(A1, A2, A3, A4, A5, A6);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);
impl_native_traits!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17);
impl_native_traits!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18
);
impl_native_traits!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19
);
impl_native_traits!(
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20
);
