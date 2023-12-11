use crate::errors::RuntimeError;
use crate::externals::function::{HostFunction, HostFunctionKind, WithEnv, WithoutEnv};
use crate::function_env::{FunctionEnv, FunctionEnvMut};
use crate::jsc::as_js::{param_from_js, AsJs};
use crate::jsc::engine::JSC;
use crate::jsc::store::{InternalStoreHandle, StoreHandle};
use crate::jsc::trap::Trap;
use crate::jsc::vm::{VMExtern, VMFuncRef, VMFunction, VMFunctionCallback, VMFunctionEnvironment};
use crate::native_type::{FromToNativeWasmType, IntoResult, NativeWasmTypeInto, WasmTypeList};
use crate::store::{AsStoreMut, AsStoreRef, StoreMut};
use crate::value::Value;
use std::fmt;
use std::marker::PhantomData;
use std::panic::{self, AssertUnwindSafe};

use wasmer_types::{FunctionType, RawValue};

use rusty_jsc::{
    callback, callback_closure, JSContext, JSObject, JSObjectCallAsFunctionCallback, JSValue,
};

#[derive(Clone, PartialEq)]
pub struct Function {
    pub(crate) handle: VMFunction,
}

// Function can't be Send in js because it dosen't support `structuredClone`
// https://developer.mozilla.org/en-US/docs/Web/API/structuredClone
// unsafe impl Send for Function {}

impl From<VMFunction> for Function {
    fn from(handle: VMFunction) -> Self {
        Self { handle }
    }
}

impl Function {
    /// To `VMExtern`.
    pub fn to_vm_extern(&self) -> VMExtern {
        VMExtern::Function(self.handle.clone())
    }

    #[allow(clippy::cast_ptr_alignment)]
    pub fn new_with_env<FT, F, T: Send + 'static>(
        store: &mut impl AsStoreMut,
        env: &FunctionEnv<T>,
        ty: FT,
        func: F,
    ) -> Self
    where
        FT: Into<FunctionType>,
        F: Fn(FunctionEnvMut<'_, T>, &[Value]) -> Result<Vec<Value>, RuntimeError>
            + 'static
            + Send
            + Sync,
    {
        let store = store.as_store_mut();
        let context = store.jsc().context();
        let function_type = ty.into();

        let new_function_type = function_type.clone();
        let raw_env = env.clone();

        let callback = callback_closure!(&context, move |ctx: JSContext,
                                                         function: JSObject,
                                                         this: JSObject,
                                                         args: &[JSValue]|
              -> Result<JSValue, JSValue> {
            let global = ctx.get_global_object();
            let store_ptr = global
                .get_property(&ctx, "__store_ptr".to_string())
                .to_number(&ctx)
                .unwrap();

            let mut store = unsafe { StoreMut::from_raw(store_ptr as usize as *mut _) };

            let env: FunctionEnvMut<T> = raw_env.clone().into_mut(&mut store);

            let wasm_arguments = new_function_type
                .params()
                .iter()
                .enumerate()
                .map(|(i, param)| param_from_js(&ctx, param, &args[i]))
                .collect::<Vec<_>>();
            let results = func(env, &wasm_arguments).map_err(|e| {
                let value = format!("{}", e);
                JSValue::string(&ctx, value)
            })?;
            match new_function_type.results().len() {
                0 => Ok(JSValue::undefined(&ctx)),
                1 => Ok(results[0].as_jsvalue(&mut store)),
                _ => Ok(JSObject::new_array(
                    &ctx,
                    &results
                        .into_iter()
                        .map(|result| result.as_jsvalue(&mut store))
                        .collect::<Vec<_>>(),
                )?
                .to_jsvalue()),
            }
        });

        let vm_function = VMFunction::new(callback, function_type);
        Self {
            handle: vm_function,
        }
    }

    /// Creates a new host `Function` from a native function.
    pub fn new_typed<F, Args, Rets>(store: &mut impl AsStoreMut, func: F) -> Self
    where
        F: HostFunction<(), Args, Rets, WithoutEnv> + 'static + Send + Sync,
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        let store = store.as_store_mut();
        let function = WasmFunction::<Args, Rets>::new(func);
        let callback = function.callback(store.jsc().context());

        let ty = function.ty();
        let vm_function = VMFunction::new(callback, ty);
        Self {
            handle: vm_function,
        }
    }

    pub fn new_typed_with_env<T, F, Args, Rets>(
        store: &mut impl AsStoreMut,
        env: &FunctionEnv<T>,
        func: F,
    ) -> Self
    where
        F: HostFunction<T, Args, Rets, WithEnv>,
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        let store = store.as_store_mut();
        let context = store.jsc().context();
        let function = WasmFunction::<Args, Rets>::new(func);
        let callback = function.callback(store.jsc().context());

        let bind = callback
            .get_property(&context, "bind".to_string())
            .to_object(&context)
            .unwrap();
        let callback_with_env = bind
            .call(
                &context,
                Some(&callback),
                &[
                    JSValue::undefined(&context),
                    JSValue::number(&context, env.handle.internal_handle().index() as f64),
                ],
            )
            .unwrap()
            .to_object(&context)
            .unwrap();

        let ty = function.ty();
        let vm_function = VMFunction::new(callback_with_env, ty);
        Self {
            handle: vm_function,
        }
    }

    pub fn ty(&self, _store: &impl AsStoreRef) -> FunctionType {
        self.handle.ty.clone()
    }

    pub fn call_raw(
        &self,
        _store: &mut impl AsStoreMut,
        _params: Vec<RawValue>,
    ) -> Result<Box<[Value]>, RuntimeError> {
        // There is no optimal call_raw in JSC, so we just
        // simply rely the call
        // self.call(store, params)
        unimplemented!();
    }

    pub fn call(
        &self,
        store: &mut impl AsStoreMut,
        params: &[Value],
    ) -> Result<Box<[Value]>, RuntimeError> {
        let store_mut = store.as_store_mut();
        let engine = store_mut.engine();
        let context = engine.0.context();

        let mut global = context.get_global_object();
        let store_ptr = store_mut.as_raw() as usize;
        global.set_property(
            &context,
            "__store_ptr".to_string(),
            JSValue::number(&context, store_ptr as _),
        );

        let params_list = params
            .iter()
            .map(|v| v.as_jsvalue(&store_mut))
            .collect::<Vec<_>>();
        let result = {
            let mut r;
            // TODO: This loop is needed for asyncify. It will be refactored with https://github.com/wasmerio/wasmer/issues/3451
            loop {
                let store_mut = store.as_store_mut();
                let engine = store_mut.engine();
                let context = engine.0.context();
                r = self.handle.function.call(&context, None, &params_list);
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
            r?
        };
        let result_types = self.handle.ty.results();
        let store_mut = store.as_store_mut();
        let engine = store_mut.engine();
        let context = engine.0.context();
        match result_types.len() {
            0 => Ok(Box::new([])),
            1 => {
                let value = param_from_js(&context, &result_types[0], &result);
                Ok(vec![value].into_boxed_slice())
            }
            n => {
                let result = result.to_object(&context).unwrap();
                Ok((0..n)
                    .map(|i| {
                        let js_val = result.get_property_at_index(&context, i as _).unwrap();
                        param_from_js(&context, &result_types[i], &js_val)
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice())
            }
        }
    }

    pub(crate) fn from_vm_extern(_store: &mut impl AsStoreMut, internal: VMFunction) -> Self {
        Self { handle: internal }
    }

    pub(crate) fn vm_funcref(&self, _store: &impl AsStoreRef) -> VMFuncRef {
        unimplemented!();
    }

    pub(crate) unsafe fn from_vm_funcref(
        _store: &mut impl AsStoreMut,
        _funcref: VMFuncRef,
    ) -> Self {
        unimplemented!();
    }

    /// Checks whether this `Function` can be used with the given context.
    pub fn is_from_store(&self, _store: &impl AsStoreRef) -> bool {
        true
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("Function").finish()
    }
}

/// Represents a low-level Wasm static host function. See
/// `super::Function::new` and `super::Function::new_env` to learn
/// more.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct WasmFunction<Args = (), Rets = ()> {
    callback: JSObjectCallAsFunctionCallback,
    _phantom: PhantomData<(Args, Rets)>,
}

unsafe impl<Args, Rets> Send for WasmFunction<Args, Rets> {}

impl<Args, Rets> WasmFunction<Args, Rets>
where
    Args: WasmTypeList,
    Rets: WasmTypeList,
{
    /// Creates a new `WasmFunction`.
    #[allow(dead_code)]
    pub fn new<F, T, Kind: HostFunctionKind>(function: F) -> Self
    where
        F: HostFunction<T, Args, Rets, Kind>,
        T: Sized,
    {
        Self {
            callback: function.function_callback(),
            _phantom: PhantomData,
        }
    }

    /// Get the function type of this `WasmFunction`.
    #[allow(dead_code)]
    pub fn ty(&self) -> FunctionType {
        FunctionType::new(Args::wasm_types(), Rets::wasm_types())
    }

    /// Get the address of this `WasmFunction`.
    #[allow(dead_code)]
    pub fn callback(&self, context: &JSContext) -> JSObject {
        JSObject::new_function_with_callback(context, "FunctionCallback".to_string(), self.callback)
    }
}

macro_rules! impl_host_function {
        ( [$c_struct_representation:ident]
           $c_struct_name:ident,
           $( $x:ident ),* ) => {

            // Implement `HostFunction` for a function with a [`FunctionEnvMut`] that has the same
            // arity than the tuple.
            #[allow(unused_parens)]
            impl< $( $x, )* Rets, RetsAsResult, T, Func >
                HostFunction<T, ( $( $x ),* ), Rets, WithEnv>
            for
                Func
            where
                $( $x: FromToNativeWasmType, )*
                Rets: WasmTypeList,
                RetsAsResult: IntoResult<Rets>,
                T: Send + 'static,
                Func: Fn(FunctionEnvMut<'_, T>, $( $x , )*) -> RetsAsResult + 'static,
            {
                #[allow(non_snake_case)]
                fn function_callback(&self) -> VMFunctionCallback {
                    #[callback]
                    fn fn_callback<T, $( $x, )* Rets, RetsAsResult, Func>(
                        ctx: JSContext,
                        function: JSObject,
                        this_object: JSObject,
                        arguments: &[JSValue],
                    ) -> Result<JSValue, JSValue>
                    where
                        $( $x: FromToNativeWasmType, )*
                        Rets: WasmTypeList,
                        RetsAsResult: IntoResult<Rets>,
                        Func: Fn(FunctionEnvMut<'_, T>, $( $x , )*) -> RetsAsResult + 'static,
                        T: Send + 'static,
                    {
                        use std::convert::TryInto;

                        let func: &Func = &*(&() as *const () as *const Func);
                        let global = ctx.get_global_object();
                        let store_ptr = global.get_property(&ctx, "__store_ptr".to_string()).to_number(&ctx).unwrap();
                        if store_ptr.is_nan() {
                            panic!("Store pointer is invalid. Received {}", store_ptr as usize)
                        }
                        let mut store = StoreMut::from_raw(store_ptr as usize as *mut _);

                        let handle_index = arguments[0].to_number(&ctx).unwrap() as usize;
                        let handle: StoreHandle<VMFunctionEnvironment> = StoreHandle::from_internal(store.objects_mut().id(), InternalStoreHandle::from_index(handle_index).unwrap());
                        let env: FunctionEnvMut<T> = FunctionEnv::from_handle(handle).into_mut(&mut store);

                        let result = panic::catch_unwind(AssertUnwindSafe(|| {
                            type JSArray<'a> = &'a [JSValue; count_idents!( $( $x ),* )];
                            let args_without_store: JSArray = arguments[1..].try_into().unwrap();
                            let [ $( $x ),* ] = args_without_store;
                            let mut store = StoreMut::from_raw(store_ptr as usize as *mut _);
                            func(env, $( FromToNativeWasmType::from_native( $x::Native::from_raw(&mut store, RawValue { u128: {
                                // TODO: This may not be the fastest way, but JSC doesn't expose a BigInt interface
                                // so the only thing we can do is parse from the string repr
                                if $x.is_number(&ctx) {
                                    $x.to_number(&ctx).unwrap() as _
                                }
                                else {
                                    $x.to_string(&ctx).unwrap().to_string().parse::<u128>().unwrap()
                                }
                            } }) ) ),* ).into_result()
                        }));

                        match result {
                            Ok(Ok(result)) => {
                                match Rets::size() {
                                    0 => {Ok(JSValue::undefined(&ctx))},
                                    1 => {
                                        // unimplemented!();

                                        let ty = Rets::wasm_types()[0];
                                        let mut arr = result.into_array(&mut store);
                                        // Value::from_raw(&store, ty, arr[0])
                                        let val = Value::from_raw(&mut store, ty, arr.as_mut()[0]);
                                        let value: JSValue = val.as_jsvalue(&store);
                                        Ok(value)
                                        // *mut_rets = val.as_raw(&mut store);
                                    }
                                    _n => {
                                        // if !results.is_array(&context) {
                                        //     panic!("Expected results to be an array.")
                                        // }
                                        let mut arr = result.into_array(&mut store);
                                        let result_values = Rets::wasm_types().iter().enumerate().map(|(i, ret_type)| {
                                            let raw = arr.as_mut()[i];
                                            Value::from_raw(&mut store, *ret_type, raw).as_jsvalue(&mut store)
                                        }).collect::<Vec<_>>();
                                        Ok(JSObject::new_array(&ctx, &result_values).unwrap().to_jsvalue())
                                    }
                                }
                            },
                            #[cfg(feature = "std")]
                            Ok(Err(err)) => {
                                let trap: Trap = Trap::user(Box::new(err));
                                Err(trap.into_jsvalue(&ctx))
                            },
                            #[cfg(feature = "core")]
                            Ok(Err(err)) => {
                                let trap: Trap = Trap::user(Box::new(err));
                                Err(trap.into_jsvalue(&ctx))
                            },
                            Err(panic) => {
                                Err(JSValue::string(&ctx, format!("panic: {:?}", panic)))
                                // We can't just resume the unwind, because it will put
                                // JavacriptCore in a bad state, so we need to transform
                                // the error

                                // std::panic::resume_unwind(panic)
                            },
                        }

                    }
                    Some(fn_callback::<T, $( $x, )* Rets, RetsAsResult, Self > as _)
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

                    #[callback]
                    fn fn_callback<$( $x, )* Rets, RetsAsResult, Func>(
                        ctx: JSContext,
                        function: JSObject,
                        this_object: JSObject,
                        arguments: &[JSValue],
                    ) -> Result<JSValue, JSValue>
                    where
                        $( $x: FromToNativeWasmType, )*
                        Rets: WasmTypeList,
                        RetsAsResult: IntoResult<Rets>,
                        Func: Fn($( $x , )*) -> RetsAsResult + 'static,
                        // $( $x: NativeWasmTypeInto, )*
                    {
                        use std::convert::TryInto;

                        let func: &Func = &*(&() as *const () as *const Func);
                        let global = ctx.get_global_object();
                        let store_ptr = global.get_property(&ctx, "__store_ptr".to_string()).to_number(&ctx).unwrap();
                        if store_ptr.is_nan() {
                            panic!("Store pointer is invalid. Received {}", store_ptr as usize)
                        }

                        let mut store = StoreMut::from_raw(store_ptr as usize as *mut _);
                        let result = panic::catch_unwind(AssertUnwindSafe(|| {
                            type JSArray<'a> = &'a [JSValue; count_idents!( $( $x ),* )];
                            let args_without_store: JSArray = arguments.try_into().unwrap();
                            let [ $( $x ),* ] = args_without_store;
                            func($( FromToNativeWasmType::from_native( $x::Native::from_raw(&mut store, RawValue { u128: {
                                // TODO: This may not be the fastest way, but JSC doesn't expose a BigInt interface
                                // so the only thing we can do is parse from the string repr
                                if $x.is_number(&ctx) {
                                    $x.to_number(&ctx).unwrap() as _
                                }
                                else {
                                    $x.to_string(&ctx).unwrap().to_string().parse::<u128>().unwrap()
                                }
                            } }) ) ),* ).into_result()
                        }));

                        match result {
                            Ok(Ok(result)) => {
                                match Rets::size() {
                                    0 => {Ok(JSValue::undefined(&ctx))},
                                    1 => {
                                        let ty = Rets::wasm_types()[0];
                                        let mut arr = result.into_array(&mut store);
                                        let val = Value::from_raw(&mut store, ty, arr.as_mut()[0]);
                                        let value: JSValue = val.as_jsvalue(&store);
                                        Ok(value)
                                    }
                                    _n => {
                                        let mut arr = result.into_array(&mut store);
                                        let result_values = Rets::wasm_types().iter().enumerate().map(|(i, ret_type)| {
                                            let raw = arr.as_mut()[i];
                                            Value::from_raw(&mut store, *ret_type, raw).as_jsvalue(&mut store)
                                        }).collect::<Vec<_>>();
                                        Ok(JSObject::new_array(&ctx, &result_values).unwrap().to_jsvalue())
                                    }
                                }
                            },
                            #[cfg(feature = "std")]
                            Ok(Err(err)) => {
                                let trap: Trap = Trap::user(Box::new(err));
                                Err(trap.into_jsvalue(&ctx))
                            },
                            #[cfg(feature = "core")]
                            Ok(Err(err)) => {
                                let trap: Trap = Trap::user(Box::new(err));
                                Err(trap.into_jsvalue(&ctx))
                            },
                            Err(panic) => {
                                Err(JSValue::string(&ctx, format!("panic: {:?}", panic)))
                                // We can't just resume the unwind, because it will put
                                // JavacriptCore in a bad state, so we need to transform
                                // the error

                                // std::panic::resume_unwind(panic)
                            },
                        }

                    }
                    Some(fn_callback::< $( $x, )* Rets, RetsAsResult, Self > as _)
                }
            }
        };
    }

// Black-magic to count the number of identifiers at compile-time.
macro_rules! count_idents {
    ( $($idents:ident),* ) => {
        {
            #[allow(dead_code, non_camel_case_types)]
            enum Idents { $( $idents, )* __CountIdentsLast }
            const COUNT: usize = Idents::__CountIdentsLast as usize;
            COUNT
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
