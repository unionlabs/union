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

use crate::sys::externals::function::{DynamicFunction, VMDynamicFunction};
use crate::sys::{FromToNativeWasmType, Function, RuntimeError, Store, WasmTypeList};
use std::panic::{catch_unwind, AssertUnwindSafe};
use wasmer_engine::ExportFunction;
use wasmer_types::NativeWasmType;
use wasmer_vm::{VMDynamicFunctionContext, VMFunctionBody, VMFunctionEnvironment, VMFunctionKind};

/// A WebAssembly function that can be called natively
/// (using the Native ABI).
pub struct NativeFunc<Args = (), Rets = ()> {
    store: Store,
    exported: ExportFunction,
    _phantom: PhantomData<(Args, Rets)>,
}

unsafe impl<Args, Rets> Send for NativeFunc<Args, Rets> {}

impl<Args, Rets> NativeFunc<Args, Rets>
where
    Args: WasmTypeList,
    Rets: WasmTypeList,
{
    pub(crate) fn new(store: Store, exported: ExportFunction) -> Self {
        Self {
            store,
            exported,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn is_host(&self) -> bool {
        self.exported.vm_function.instance_ref.is_none()
    }

    pub(crate) fn vmctx(&self) -> VMFunctionEnvironment {
        self.exported.vm_function.vmctx
    }

    pub(crate) fn address(&self) -> *const VMFunctionBody {
        self.exported.vm_function.address
    }

    pub(crate) fn arg_kind(&self) -> VMFunctionKind {
        self.exported.vm_function.kind
    }

    /// Get access to the backing VM value for this extern. This function is for
    /// tests it should not be called by users of the Wasmer API.
    ///
    /// # Safety
    /// This function is unsafe to call outside of tests for the wasmer crate
    /// because there is no stability guarantee for the returned type and we may
    /// make breaking changes to it at any time or remove this method.
    #[doc(hidden)]
    pub unsafe fn get_vm_function(&self) -> &wasmer_vm::VMFunction {
        &self.exported.vm_function
    }
}

/*
impl<Args, Rets> From<&NativeFunc<Args, Rets>> for VMFunction
where
    Args: WasmTypeList,
    Rets: WasmTypeList,
{
    fn from(other: &NativeFunc<Args, Rets>) -> Self {
        let signature = FunctionType::new(Args::wasm_types(), Rets::wasm_types());
        Self {
            address: other.address,
            vmctx: other.vmctx,
            signature,
            kind: other.arg_kind,
            call_trampoline: None,
            instance_ref: None,
        }
    }
}*/

impl<Args: WasmTypeList, Rets: WasmTypeList> Clone for NativeFunc<Args, Rets> {
    fn clone(&self) -> Self {
        let mut exported = self.exported.clone();
        exported.vm_function.upgrade_instance_ref().unwrap();

        Self {
            store: self.store.clone(),
            exported,
            _phantom: PhantomData,
        }
    }
}

impl<Args, Rets> From<&NativeFunc<Args, Rets>> for ExportFunction
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
                if !self.is_host() {
                    // We assume the trampoline is always going to be present for
                    // Wasm functions
                    let trampoline = self.exported.vm_function.call_trampoline.expect("Call trampoline not found in wasm function");
                    // TODO: when `const fn` related features mature more, we can declare a single array
                    // of the correct size here.
                    let mut params_list = [ $( $x.to_native().to_binary() ),* ];
                    let mut rets_list_array = Rets::empty_array();
                    let rets_list = rets_list_array.as_mut();
                    let using_rets_array;
                    let args_rets: &mut [i128] = if params_list.len() > rets_list.len() {
                        using_rets_array = false;
                        params_list.as_mut()
                    } else {
                        using_rets_array = true;
                        for (i, &arg) in params_list.iter().enumerate() {
                            rets_list[i] = arg;
                        }
                        rets_list.as_mut()
                    };
                    unsafe {
                        wasmer_vm::wasmer_call_trampoline(
                            &self.store,
                            self.vmctx(),
                            trampoline,
                            self.address(),
                            args_rets.as_mut_ptr() as *mut u8,
                        )
                    }?;
                    let num_rets = rets_list.len();
                    if !using_rets_array && num_rets > 0 {
                        let src_pointer = params_list.as_ptr();
                        let rets_list = &mut rets_list_array.as_mut()[0] as *mut i128;
                        unsafe {
                            // TODO: we can probably remove this copy by doing some clever `transmute`s.
                            // we know it's not overlapping because `using_rets_array` is false
                            std::ptr::copy_nonoverlapping(src_pointer,
                                                          rets_list,
                                                          num_rets);
                        }
                    }
                    Ok(Rets::from_array(rets_list_array))
                    // TODO: When the Host ABI and Wasm ABI are the same, we could do this instead:
                    // but we can't currently detect whether that's safe.
                    //
                    // let results = unsafe {
                    //     wasmer_vm::catch_traps_with_result(self.vmctx, || {
                    //         let f = std::mem::transmute::<_, unsafe extern "C" fn( *mut VMContext, $( $x, )*) -> Rets::CStruct>(self.address());
                    //         // We always pass the vmctx
                    //         f( self.vmctx, $( $x, )* )
                    //     }).map_err(RuntimeError::from_trap)?
                    // };
                    // Ok(Rets::from_c_struct(results))

                }
                else {
                    match self.arg_kind() {
                        VMFunctionKind::Static => {
                            let results = catch_unwind(AssertUnwindSafe(|| unsafe {
                                let f = std::mem::transmute::<_, unsafe extern "C" fn( VMFunctionEnvironment, $( $x, )*) -> Rets::CStruct>(self.address());
                                // We always pass the vmctx
                                f( self.vmctx(), $( $x, )* )
                            })).map_err(|e| RuntimeError::new(format!("{:?}", e)))?;
                            Ok(Rets::from_c_struct(results))
                        },
                        VMFunctionKind::Dynamic => {
                            let params_list = [ $( $x.to_native().to_value() ),* ];
                            let results = {
                                type VMContextWithEnv = VMDynamicFunctionContext<DynamicFunction<std::ffi::c_void>>;
                                unsafe {
                                    let ctx = self.vmctx().host_env as *mut VMContextWithEnv;
                                    (*ctx).ctx.call(&params_list)?
                                }
                            };
                            let mut rets_list_array = Rets::empty_array();
                            let mut_rets = rets_list_array.as_mut() as *mut [i128] as *mut i128;
                            for (i, ret) in results.iter().enumerate() {
                                unsafe {
                                    ret.write_value_to(mut_rets.add(i));
                                }
                            }
                            Ok(Rets::from_array(rets_list_array))
                        }
                    }
                }
            }

        }

        #[allow(unused_parens)]
        impl<'a, $( $x, )* Rets> crate::sys::exports::ExportableWithGenerics<'a, ($( $x ),*), Rets> for NativeFunc<( $( $x ),* ), Rets>
        where
            $( $x: FromToNativeWasmType, )*
            Rets: WasmTypeList,
        {
            fn get_self_from_extern_with_generics(_extern: &crate::sys::externals::Extern) -> Result<Self, crate::sys::exports::ExportError> {
                use crate::sys::exports::Exportable;
                crate::Function::get_self_from_extern(_extern)?.native().map_err(|_| crate::sys::exports::ExportError::IncompatibleType)
            }

            fn into_weak_instance_ref(&mut self) {
                self.exported.vm_function.instance_ref.as_mut().map(|v| *v = v.downgrade());
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
