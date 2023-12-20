use crate::{FromToNativeWasmType, RuntimeError, TypedFunction, WasmTypeList};
use wasmer_types::RawValue;

use crate::native_type::NativeWasmTypeInto;
use crate::store::{AsStoreMut, AsStoreRef};
use crate::sys::engine::NativeEngineExt;

macro_rules! impl_native_traits {
    (  $( $x:ident ),* ) => {
        #[allow(unused_parens, non_snake_case)]
        impl<$( $x , )* Rets> TypedFunction<( $( $x ),* ), Rets>
        where
            $( $x: FromToNativeWasmType, )*
            Rets: WasmTypeList,
        {
            /// Call the typed func and return results.
            #[allow(unused_mut)]
            #[allow(clippy::too_many_arguments)]
            pub fn call(&self, store: &mut impl AsStoreMut, $( $x: $x, )* ) -> Result<Rets, RuntimeError> {
                let anyfunc = unsafe {
                    *self.func.0
                        .handle
                        .get(store.as_store_ref().objects())
                        .anyfunc
                        .as_ptr()
                        .as_ref()
                };
                // Ensure all parameters come from the same context.
                if $(!FromToNativeWasmType::is_from_store(&$x, store) ||)* false {
                    return Err(RuntimeError::new(
                        "cross-`Store` values are not supported",
                    ));
                }
                // TODO: when `const fn` related features mature more, we can declare a single array
                // of the correct size here.
                let mut params_list = [ $( $x.to_native().into_raw(store) ),* ];
                let mut rets_list_array = Rets::empty_array();
                let rets_list: &mut [RawValue] = rets_list_array.as_mut();
                let using_rets_array;
                let args_rets: &mut [RawValue] = if params_list.len() > rets_list.len() {
                    using_rets_array = false;
                    params_list.as_mut()
                } else {
                    using_rets_array = true;
                    for (i, &arg) in params_list.iter().enumerate() {
                        rets_list[i] = arg;
                    }
                    rets_list.as_mut()
                };

                let mut r;
                loop {
                    let storeref = store.as_store_ref();
                    let config = storeref.engine().tunables().vmconfig();
                    r = unsafe {
                        wasmer_vm::wasmer_call_trampoline(
                            store.as_store_ref().signal_handler(),
                            config,
                            anyfunc.vmctx,
                            anyfunc.call_trampoline,
                            anyfunc.func_ptr,
                            args_rets.as_mut_ptr() as *mut u8,
                        )
                    };
                    let store_mut = store.as_store_mut();
                    if let Some(callback) = store_mut.inner.on_called.take() {
                        match callback(store_mut) {
                            Ok(wasmer_types::OnCalledAction::InvokeAgain) => { continue; }
                            Ok(wasmer_types::OnCalledAction::Finish) => { break; }
                            Ok(wasmer_types::OnCalledAction::Trap(trap)) => { return Err(RuntimeError::user(trap)) },
                            Err(trap) => { return Err(RuntimeError::user(trap)) },
                        }
                    }
                    break;
                }
                r?;

                let num_rets = rets_list.len();
                if !using_rets_array && num_rets > 0 {
                    let src_pointer = params_list.as_ptr();
                    let rets_list = &mut rets_list_array.as_mut()[0] as *mut RawValue;
                    unsafe {
                        // TODO: we can probably remove this copy by doing some clever `transmute`s.
                        // we know it's not overlapping because `using_rets_array` is false
                        std::ptr::copy_nonoverlapping(src_pointer,
                                                        rets_list,
                                                        num_rets);
                    }
                }
                Ok(unsafe { Rets::from_array(store, rets_list_array) })
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

            #[doc(hidden)]
            #[allow(missing_docs)]
            #[allow(unused_mut)]
            #[allow(clippy::too_many_arguments)]
            pub fn call_raw(&self, store: &mut impl AsStoreMut, mut params_list: Vec<RawValue> ) -> Result<Rets, RuntimeError> {
                let anyfunc = unsafe {
                    *self.func.0
                        .handle
                        .get(store.as_store_ref().objects())
                        .anyfunc
                        .as_ptr()
                        .as_ref()
                };
                // TODO: when `const fn` related features mature more, we can declare a single array
                // of the correct size here.
                let mut rets_list_array = Rets::empty_array();
                let rets_list: &mut [RawValue] = rets_list_array.as_mut();
                let using_rets_array;
                let args_rets: &mut [RawValue] = if params_list.len() > rets_list.len() {
                    using_rets_array = false;
                    params_list.as_mut()
                } else {
                    using_rets_array = true;
                    for (i, &arg) in params_list.iter().enumerate() {
                        rets_list[i] = arg;
                    }
                    rets_list.as_mut()
                };

                let mut r;
                loop {
                    let storeref = store.as_store_ref();
                    let config = storeref.engine().tunables().vmconfig();
                    r = unsafe {
                        wasmer_vm::wasmer_call_trampoline(
                            store.as_store_ref().signal_handler(),
                            config,
                            anyfunc.vmctx,
                            anyfunc.call_trampoline,
                            anyfunc.func_ptr,
                            args_rets.as_mut_ptr() as *mut u8,
                        )
                    };
                    let store_mut = store.as_store_mut();
                    if let Some(callback) = store_mut.inner.on_called.take() {
                        // TODO: OnCalledAction is needed for asyncify. It will be refactored with https://github.com/wasmerio/wasmer/issues/3451
                        match callback(store_mut) {
                            Ok(wasmer_types::OnCalledAction::InvokeAgain) => { continue; }
                            Ok(wasmer_types::OnCalledAction::Finish) => { break; }
                            Ok(wasmer_types::OnCalledAction::Trap(trap)) => { return Err(RuntimeError::user(trap)) },
                            Err(trap) => { return Err(RuntimeError::user(trap)) },
                        }
                    }
                    break;
                }
                r?;

                let num_rets = rets_list.len();
                if !using_rets_array && num_rets > 0 {
                    let src_pointer = params_list.as_ptr();
                    let rets_list = &mut rets_list_array.as_mut()[0] as *mut RawValue;
                    unsafe {
                        // TODO: we can probably remove this copy by doing some clever `transmute`s.
                        // we know it's not overlapping because `using_rets_array` is false
                        std::ptr::copy_nonoverlapping(src_pointer,
                                                        rets_list,
                                                        num_rets);
                    }
                }
                Ok(unsafe { Rets::from_array(store, rets_list_array) })
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
