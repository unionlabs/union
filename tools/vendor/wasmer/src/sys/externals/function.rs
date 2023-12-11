use crate::externals::function::{HostFunction, WithEnv, WithoutEnv};
use crate::native_type::{FromToNativeWasmType, IntoResult, NativeWasmTypeInto, WasmTypeList};
use crate::store::{AsStoreMut, AsStoreRef, StoreInner, StoreMut};
use crate::sys::engine::NativeEngineExt;
use crate::vm::{VMExternFunction, VMFunctionCallback};
use crate::{FunctionEnv, FunctionEnvMut, FunctionType, RuntimeError, Value};
use std::panic::{self, AssertUnwindSafe};
use std::{cell::UnsafeCell, cmp::max, ffi::c_void};
use wasmer_types::{NativeWasmType, RawValue};
use wasmer_vm::{
    on_host_stack, raise_user_trap, resume_panic, wasmer_call_trampoline, MaybeInstanceOwned,
    StoreHandle, VMCallerCheckedAnyfunc, VMContext, VMDynamicFunctionContext, VMExtern, VMFuncRef,
    VMFunction, VMFunctionContext, VMFunctionKind, VMTrampoline,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub(crate) handle: StoreHandle<VMFunction>,
}

impl From<StoreHandle<VMFunction>> for Function {
    fn from(handle: StoreHandle<VMFunction>) -> Self {
        Self { handle }
    }
}

impl Function {
    pub fn new_with_env<FT, F, T: Send + 'static>(
        store: &mut impl AsStoreMut,
        env: &FunctionEnv<T>,
        ty: FT,
        func: F,
    ) -> Self
    where
        FT: Into<FunctionType>,
        F: Fn(FunctionEnvMut<T>, &[Value]) -> Result<Vec<Value>, RuntimeError>
            + 'static
            + Send
            + Sync,
    {
        let function_type = ty.into();
        let func_ty = function_type.clone();
        let func_env = env.clone();
        let raw_store = store.as_store_mut().as_raw() as *mut u8;
        let wrapper = move |values_vec: *mut RawValue| -> Result<(), RuntimeError> {
            unsafe {
                let mut store = StoreMut::from_raw(raw_store as *mut StoreInner);
                let mut args = Vec::with_capacity(func_ty.params().len());
                for (i, ty) in func_ty.params().iter().enumerate() {
                    args.push(Value::from_raw(&mut store, *ty, *values_vec.add(i)));
                }
                let store_mut = StoreMut::from_raw(raw_store as *mut StoreInner);
                let env = FunctionEnvMut {
                    store_mut,
                    func_env: func_env.clone(),
                };
                let returns = func(env, &args)?;

                // We need to dynamically check that the returns
                // match the expected types, as well as expected length.
                let return_types = returns.iter().map(|ret| ret.ty());
                if return_types.ne(func_ty.results().iter().copied()) {
                    return Err(RuntimeError::new(format!(
                        "Dynamic function returned wrong signature. Expected {:?} but got {:?}",
                        func_ty.results(),
                        returns.iter().map(|ret| ret.ty())
                    )));
                }
                for (i, ret) in returns.iter().enumerate() {
                    *values_vec.add(i) = ret.as_raw(&store);
                }
            }
            Ok(())
        };
        let mut host_data = Box::new(VMDynamicFunctionContext {
            address: std::ptr::null(),
            ctx: DynamicFunction { func: wrapper },
        });
        host_data.address = host_data.ctx.func_body_ptr();

        // We don't yet have the address with the Wasm ABI signature.
        // The engine linker will replace the address with one pointing to a
        // generated dynamic trampoline.
        let func_ptr = std::ptr::null() as VMFunctionCallback;
        let type_index = store
            .as_store_mut()
            .engine()
            .0
            .register_signature(&function_type);
        let vmctx = VMFunctionContext {
            host_env: host_data.as_ref() as *const _ as *mut c_void,
        };
        let call_trampoline = host_data.ctx.call_trampoline_address();
        let anyfunc = VMCallerCheckedAnyfunc {
            func_ptr,
            type_index,
            vmctx,
            call_trampoline,
        };

        let vm_function = VMFunction {
            anyfunc: MaybeInstanceOwned::Host(Box::new(UnsafeCell::new(anyfunc))),
            kind: VMFunctionKind::Dynamic,
            signature: function_type,
            host_data,
        };
        Self {
            handle: StoreHandle::new(store.as_store_mut().objects_mut(), vm_function),
        }
    }

    /// Creates a new host `Function` from a native function.
    pub fn new_typed<F, Args, Rets>(store: &mut impl AsStoreMut, func: F) -> Self
    where
        F: HostFunction<(), Args, Rets, WithoutEnv> + 'static + Send + Sync,
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        let env = FunctionEnv::new(store, ());
        let func_ptr = func.function_callback();
        let host_data = Box::new(StaticFunction {
            raw_store: store.as_store_mut().as_raw() as *mut u8,
            env,
            func,
        });
        let function_type = FunctionType::new(Args::wasm_types(), Rets::wasm_types());

        let type_index = store
            .as_store_mut()
            .engine()
            .0
            .register_signature(&function_type);
        let vmctx = VMFunctionContext {
            host_env: host_data.as_ref() as *const _ as *mut c_void,
        };
        let call_trampoline =
            <F as HostFunction<(), Args, Rets, WithoutEnv>>::call_trampoline_address();
        let anyfunc = VMCallerCheckedAnyfunc {
            func_ptr,
            type_index,
            vmctx,
            call_trampoline,
        };

        let vm_function = VMFunction {
            anyfunc: MaybeInstanceOwned::Host(Box::new(UnsafeCell::new(anyfunc))),
            kind: VMFunctionKind::Static,
            signature: function_type,
            host_data,
        };
        Self {
            handle: StoreHandle::new(store.as_store_mut().objects_mut(), vm_function),
        }
    }

    pub fn new_typed_with_env<T: Send + 'static, F, Args, Rets>(
        store: &mut impl AsStoreMut,
        env: &FunctionEnv<T>,
        func: F,
    ) -> Self
    where
        F: HostFunction<T, Args, Rets, WithEnv> + 'static + Send + Sync,
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        let func_ptr = func.function_callback();
        let host_data = Box::new(StaticFunction {
            raw_store: store.as_store_mut().as_raw() as *mut u8,
            env: env.clone(),
            func,
        });
        let function_type = FunctionType::new(Args::wasm_types(), Rets::wasm_types());

        let type_index = store
            .as_store_mut()
            .engine()
            .0
            .register_signature(&function_type);
        let vmctx = VMFunctionContext {
            host_env: host_data.as_ref() as *const _ as *mut c_void,
        };
        let call_trampoline =
            <F as HostFunction<T, Args, Rets, WithEnv>>::call_trampoline_address();
        let anyfunc = VMCallerCheckedAnyfunc {
            func_ptr,
            type_index,
            vmctx,
            call_trampoline,
        };

        let vm_function = VMFunction {
            anyfunc: MaybeInstanceOwned::Host(Box::new(UnsafeCell::new(anyfunc))),
            kind: VMFunctionKind::Static,
            signature: function_type,
            host_data,
        };
        Self {
            handle: StoreHandle::new(store.as_store_mut().objects_mut(), vm_function),
        }
    }

    pub fn ty(&self, store: &impl AsStoreRef) -> FunctionType {
        self.handle
            .get(store.as_store_ref().objects())
            .signature
            .clone()
    }

    fn call_wasm(
        &self,
        store: &mut impl AsStoreMut,
        trampoline: VMTrampoline,
        params: &[Value],
        results: &mut [Value],
    ) -> Result<(), RuntimeError> {
        let format_types_for_error_message = |items: &[Value]| {
            items
                .iter()
                .map(|param| param.ty().to_string())
                .collect::<Vec<String>>()
                .join(", ")
        };
        // TODO: Avoid cloning the signature here, it's expensive.
        let signature = self.ty(store);
        if signature.params().len() != params.len() {
            return Err(RuntimeError::new(format!(
                "Parameters of type [{}] did not match signature {}",
                format_types_for_error_message(params),
                &signature
            )));
        }
        if signature.results().len() != results.len() {
            return Err(RuntimeError::new(format!(
                "Results of type [{}] did not match signature {}",
                format_types_for_error_message(results),
                &signature,
            )));
        }

        let mut values_vec = vec![RawValue { i32: 0 }; max(params.len(), results.len())];

        // Store the argument values into `values_vec`.
        let param_tys = signature.params().iter();
        for ((arg, slot), ty) in params.iter().zip(&mut values_vec).zip(param_tys) {
            if arg.ty() != *ty {
                let param_types = format_types_for_error_message(params);
                return Err(RuntimeError::new(format!(
                    "Parameters of type [{}] did not match signature {}",
                    param_types, &signature,
                )));
            }
            if !arg.is_from_store(store) {
                return Err(RuntimeError::new("cross-`Store` values are not supported"));
            }
            *slot = arg.as_raw(store);
        }

        // Invoke the call
        self.call_wasm_raw(store, trampoline, values_vec, results)?;
        Ok(())
    }

    fn call_wasm_raw(
        &self,
        store: &mut impl AsStoreMut,
        trampoline: VMTrampoline,
        mut params: Vec<RawValue>,
        results: &mut [Value],
    ) -> Result<(), RuntimeError> {
        // Call the trampoline.
        let result = {
            let mut r;
            // TODO: This loop is needed for asyncify. It will be refactored with https://github.com/wasmerio/wasmer/issues/3451
            loop {
                let storeref = store.as_store_ref();
                let vm_function = self.handle.get(storeref.objects());
                let config = storeref.engine().tunables().vmconfig();
                r = unsafe {
                    wasmer_call_trampoline(
                        store.as_store_ref().signal_handler(),
                        config,
                        vm_function.anyfunc.as_ptr().as_ref().vmctx,
                        trampoline,
                        vm_function.anyfunc.as_ptr().as_ref().func_ptr,
                        params.as_mut_ptr() as *mut u8,
                    )
                };
                let store_mut = store.as_store_mut();
                if let Some(callback) = store_mut.inner.on_called.take() {
                    match callback(store_mut) {
                        Ok(wasmer_types::OnCalledAction::InvokeAgain) => {
                            continue;
                        }
                        Ok(wasmer_types::OnCalledAction::Finish) => {
                            break;
                        }
                        Ok(wasmer_types::OnCalledAction::Trap(trap)) => {
                            return Err(RuntimeError::user(trap))
                        }
                        Err(trap) => return Err(RuntimeError::user(trap)),
                    }
                }
                break;
            }
            r
        };
        if let Err(error) = result {
            return Err(error.into());
        }

        // Load the return values out of `values_vec`.
        let signature = self.ty(store);
        for (index, &value_type) in signature.results().iter().enumerate() {
            unsafe {
                results[index] = Value::from_raw(store, value_type, params[index]);
            }
        }

        Ok(())
    }

    pub fn result_arity(&self, store: &impl AsStoreRef) -> usize {
        self.ty(store).results().len()
    }

    pub fn call(
        &self,
        store: &mut impl AsStoreMut,
        params: &[Value],
    ) -> Result<Box<[Value]>, RuntimeError> {
        let trampoline = unsafe {
            self.handle
                .get(store.as_store_ref().objects())
                .anyfunc
                .as_ptr()
                .as_ref()
                .call_trampoline
        };
        let mut results = vec![Value::null(); self.result_arity(store)];
        self.call_wasm(store, trampoline, params, &mut results)?;
        Ok(results.into_boxed_slice())
    }

    #[doc(hidden)]
    #[allow(missing_docs)]
    pub fn call_raw(
        &self,
        store: &mut impl AsStoreMut,
        params: Vec<RawValue>,
    ) -> Result<Box<[Value]>, RuntimeError> {
        let trampoline = unsafe {
            self.handle
                .get(store.as_store_ref().objects())
                .anyfunc
                .as_ptr()
                .as_ref()
                .call_trampoline
        };
        let mut results = vec![Value::null(); self.result_arity(store)];
        self.call_wasm_raw(store, trampoline, params, &mut results)?;
        Ok(results.into_boxed_slice())
    }

    pub(crate) fn vm_funcref(&self, store: &impl AsStoreRef) -> VMFuncRef {
        let vm_function = self.handle.get(store.as_store_ref().objects());
        if vm_function.kind == VMFunctionKind::Dynamic {
            panic!("dynamic functions cannot be used in tables or as funcrefs");
        }
        VMFuncRef(vm_function.anyfunc.as_ptr())
    }

    pub(crate) unsafe fn from_vm_funcref(store: &mut impl AsStoreMut, funcref: VMFuncRef) -> Self {
        let signature = store
            .as_store_ref()
            .engine()
            .0
            .lookup_signature(funcref.0.as_ref().type_index)
            .expect("Signature not found in store");
        let vm_function = VMFunction {
            anyfunc: MaybeInstanceOwned::Instance(funcref.0),
            signature,
            // All functions in tables are already Static (as dynamic functions
            // are converted to use the trampolines with static signatures).
            kind: wasmer_vm::VMFunctionKind::Static,
            host_data: Box::new(()),
        };
        Self {
            handle: StoreHandle::new(store.as_store_mut().objects_mut(), vm_function),
        }
    }

    pub(crate) fn from_vm_extern(store: &mut impl AsStoreMut, vm_extern: VMExternFunction) -> Self {
        Self {
            handle: unsafe {
                StoreHandle::from_internal(store.as_store_ref().objects().id(), vm_extern)
            },
        }
    }

    /// Checks whether this `Function` can be used with the given store.
    pub fn is_from_store(&self, store: &impl AsStoreRef) -> bool {
        self.handle.store_id() == store.as_store_ref().objects().id()
    }

    pub(crate) fn to_vm_extern(&self) -> VMExtern {
        VMExtern::Function(self.handle.internal_handle())
    }
}

/// Host state for a dynamic function.
pub(crate) struct DynamicFunction<F> {
    func: F,
}

impl<F> DynamicFunction<F>
where
    F: Fn(*mut RawValue) -> Result<(), RuntimeError> + 'static,
{
    // This function wraps our func, to make it compatible with the
    // reverse trampoline signature
    unsafe extern "C" fn func_wrapper(
        this: &mut VMDynamicFunctionContext<Self>,
        values_vec: *mut RawValue,
    ) {
        let result =
            on_host_stack(|| panic::catch_unwind(AssertUnwindSafe(|| (this.ctx.func)(values_vec))));

        match result {
            Ok(Ok(())) => {}
            Ok(Err(trap)) => raise_user_trap(Box::new(trap)),
            Err(panic) => resume_panic(panic),
        }
    }

    fn func_body_ptr(&self) -> VMFunctionCallback {
        Self::func_wrapper as VMFunctionCallback
    }

    fn call_trampoline_address(&self) -> VMTrampoline {
        Self::call_trampoline
    }

    unsafe extern "C" fn call_trampoline(
        vmctx: *mut VMContext,
        _body: VMFunctionCallback,
        args: *mut RawValue,
    ) {
        // The VMFunctionCallback is null here: it is only filled in later
        // by the engine linker.
        let dynamic_function = &mut *(vmctx as *mut VMDynamicFunctionContext<Self>);
        Self::func_wrapper(dynamic_function, args);
    }
}

/// Represents a low-level Wasm static host function. See
/// [`crate::Function::new_typed`] and
/// [`crate::Function::new_typed_with_env`] to learn more.
pub(crate) struct StaticFunction<F, T> {
    pub(crate) raw_store: *mut u8,
    pub(crate) env: FunctionEnv<T>,
    pub(crate) func: F,
}

macro_rules! impl_host_function {
        ( [$c_struct_representation:ident]
           $c_struct_name:ident,
           $( $x:ident ),* ) => {

            // Implement `HostFunction` for a function with a [`FunctionEnvMut`] that has the same
            // arity than the tuple.
            #[allow(unused_parens)]
            impl< $( $x, )* Rets, RetsAsResult, T: Send + 'static, Func >
                HostFunction<T, ( $( $x ),* ), Rets, WithEnv>
            for
                Func
            where
                $( $x: FromToNativeWasmType, )*
                Rets: WasmTypeList,
                RetsAsResult: IntoResult<Rets>,
                Func: Fn(FunctionEnvMut<T>, $( $x , )*) -> RetsAsResult + 'static,
            {
                #[allow(non_snake_case)]
                fn function_callback(&self) -> VMFunctionCallback {
                    /// This is a function that wraps the real host
                    /// function. Its address will be used inside the
                    /// runtime.
                    unsafe extern "C" fn func_wrapper<T: Send + 'static, $( $x, )* Rets, RetsAsResult, Func>( env: &StaticFunction<Func, T>, $( $x: <$x::Native as NativeWasmType>::Abi, )* ) -> Rets::CStruct
                    where
                        $( $x: FromToNativeWasmType, )*
                        Rets: WasmTypeList,
                        RetsAsResult: IntoResult<Rets>,
                        Func: Fn(FunctionEnvMut<T>, $( $x , )*) -> RetsAsResult + 'static,
                    {
                        // println!("func wrapper");
                        let mut store = StoreMut::from_raw(env.raw_store as *mut _);
                        let result = on_host_stack(|| {
                            // println!("func wrapper1");
                            panic::catch_unwind(AssertUnwindSafe(|| {
                                $(
                                    let $x = FromToNativeWasmType::from_native(NativeWasmTypeInto::from_abi(&mut store, $x));
                                )*
                                // println!("func wrapper2 {:p}", *env.raw_env);
                                let store_mut = StoreMut::from_raw(env.raw_store as *mut _);
                                let f_env = FunctionEnvMut {
                                    store_mut,
                                    func_env: env.env.clone(),
                                };
                                // println!("func wrapper3");
                                (env.func)(f_env, $($x),* ).into_result()
                            }))
                        });

                        match result {
                            Ok(Ok(result)) => return result.into_c_struct(&mut store),
                            Ok(Err(trap)) => raise_user_trap(Box::new(trap)),
                            Err(panic) => resume_panic(panic) ,
                        }
                    }

                    func_wrapper::< T, $( $x, )* Rets, RetsAsResult, Self > as VMFunctionCallback
                }

                #[allow(non_snake_case)]
                fn call_trampoline_address() -> VMTrampoline {
                    unsafe extern "C" fn call_trampoline<
                        $( $x: FromToNativeWasmType, )*
                        Rets: WasmTypeList,
                    >(
                        vmctx: *mut VMContext,
                        body: VMFunctionCallback,
                        args: *mut RawValue,
                    ) {
                            let body: unsafe extern "C" fn(
                                vmctx: *mut VMContext,
                                $( $x: <$x::Native as NativeWasmType>::Abi, )*
                            ) -> Rets::CStruct
                                = std::mem::transmute(body);

                            let mut _n = 0;
                            $(
                                let $x = *args.add(_n).cast();
                                _n += 1;
                            )*

                            let results = body(vmctx, $( $x ),*);
                            Rets::write_c_struct_to_ptr(results, args);
                    }

                    call_trampoline::<$( $x, )* Rets>
                }

            }

            // Implement `HostFunction` for a function that has the same arity than the tuple.
            #[allow(unused_parens)]
            impl< $( $x, )* Rets, RetsAsResult, Func >
                HostFunction<(), ( $( $x ),* ), Rets, WithoutEnv>
            for
                Func
            where
                $( $x: FromToNativeWasmType, )*
                Rets: WasmTypeList,
                RetsAsResult: IntoResult<Rets>,
                Func: Fn($( $x , )*) -> RetsAsResult + 'static,
            {
                #[allow(non_snake_case)]
                fn function_callback(&self) -> VMFunctionCallback {
                    /// This is a function that wraps the real host
                    /// function. Its address will be used inside the
                    /// runtime.
                    unsafe extern "C" fn func_wrapper<$( $x, )* Rets, RetsAsResult, Func>( env: &StaticFunction<Func, ()>, $( $x: <$x::Native as NativeWasmType>::Abi, )* ) -> Rets::CStruct
                    where
                        $( $x: FromToNativeWasmType, )*
                        Rets: WasmTypeList,
                        RetsAsResult: IntoResult<Rets>,
                        Func: Fn($( $x , )*) -> RetsAsResult + 'static,
                    {
                        // println!("func wrapper");
                        let mut store = StoreMut::from_raw(env.raw_store as *mut _);
                        let result = on_host_stack(|| {
                            // println!("func wrapper1");
                            panic::catch_unwind(AssertUnwindSafe(|| {
                                $(
                                    let $x = FromToNativeWasmType::from_native(NativeWasmTypeInto::from_abi(&mut store, $x));
                                )*
                                (env.func)($($x),* ).into_result()
                            }))
                        });

                        match result {
                            Ok(Ok(result)) => return result.into_c_struct(&mut store),
                            Ok(Err(trap)) => raise_user_trap(Box::new(trap)),
                            Err(panic) => resume_panic(panic) ,
                        }
                    }

                    func_wrapper::< $( $x, )* Rets, RetsAsResult, Self > as VMFunctionCallback
                }

                #[allow(non_snake_case)]
                fn call_trampoline_address() -> VMTrampoline {
                    unsafe extern "C" fn call_trampoline<
                        $( $x: FromToNativeWasmType, )*
                        Rets: WasmTypeList,
                    >(
                        vmctx: *mut VMContext,
                        body: VMFunctionCallback,
                        args: *mut RawValue,
                    ) {
                            let body: unsafe extern "C" fn(
                                vmctx: *mut VMContext,
                                $( $x: <$x::Native as NativeWasmType>::Abi, )*
                            ) -> Rets::CStruct
                                = std::mem::transmute(body);

                            let mut _n = 0;
                            $(
                                let $x = *args.add(_n).cast();
                                _n += 1;
                            )*

                            let results = body(vmctx, $( $x ),*);
                            Rets::write_c_struct_to_ptr(results, args);
                    }

                    call_trampoline::<$( $x, )* Rets>
                }

            }
        };
    }

// Here we go! Let's generate all the C struct, `WasmTypeList`
// implementations and `HostFunction` implementations.
impl_host_function!([C] S0,);
impl_host_function!([transparent] S1, A1);
impl_host_function!([C] S2, A1, A2);
impl_host_function!([C] S3, A1, A2, A3);
impl_host_function!([C] S4, A1, A2, A3, A4);
impl_host_function!([C] S5, A1, A2, A3, A4, A5);
impl_host_function!([C] S6, A1, A2, A3, A4, A5, A6);
impl_host_function!([C] S7, A1, A2, A3, A4, A5, A6, A7);
impl_host_function!([C] S8, A1, A2, A3, A4, A5, A6, A7, A8);
impl_host_function!([C] S9, A1, A2, A3, A4, A5, A6, A7, A8, A9);
impl_host_function!([C] S10, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
impl_host_function!([C] S11, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
impl_host_function!([C] S12, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
impl_host_function!([C] S13, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
impl_host_function!([C] S14, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
impl_host_function!([C] S15, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
impl_host_function!([C] S16, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);
impl_host_function!([C] S17, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17);
impl_host_function!([C] S18, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18);
impl_host_function!([C] S19, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19);
impl_host_function!([C] S20, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20);
impl_host_function!([C] S21, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21);
impl_host_function!([C] S22, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22);
impl_host_function!([C] S23, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23);
impl_host_function!([C] S24, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24);
impl_host_function!([C] S25, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25);
impl_host_function!([C] S26, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26);
