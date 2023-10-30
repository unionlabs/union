//! Support for long-lived closures in `wasm-bindgen`
//!
//! This module defines the `Closure` type which is used to pass "owned
//! closures" from Rust to JS. Some more details can be found on the `Closure`
//! type itself.

#![allow(clippy::fn_to_numeric_cast)]

use std::fmt;
use std::mem::{self, ManuallyDrop};
use std::prelude::v1::*;

use crate::convert::*;
use crate::describe::*;
use crate::throw_str;
use crate::JsValue;
use crate::UnwrapThrowExt;

/// A handle to both a closure in Rust as well as JS closure which will invoke
/// the Rust closure.
///
/// A `Closure` is the primary way that a `'static` lifetime closure is
/// transferred from Rust to JS. `Closure` currently requires that the closures
/// it's created with have the `'static` lifetime in Rust for soundness reasons.
///
/// This type is a "handle" in the sense that whenever it is dropped it will
/// invalidate the JS closure that it refers to. Any usage of the closure in JS
/// after the `Closure` has been dropped will raise an exception. It's then up
/// to you to arrange for `Closure` to be properly deallocate at an appropriate
/// location in your program.
///
/// The type parameter on `Closure` is the type of closure that this represents.
/// Currently this can only be the `Fn` and `FnMut` traits with up to 7
/// arguments (and an optional return value).
///
/// # Examples
///
/// Here are a number of examples of using `Closure`.
///
/// ## Using the `setInterval` API
///
/// Sample usage of `Closure` to invoke the `setInterval` API.
///
/// ```rust,no_run
/// use wasm_bindgen::prelude::*;
///
/// #[wasm_bindgen]
/// extern "C" {
///     fn setInterval(closure: &Closure<dyn FnMut()>, time: u32) -> i32;
///     fn clearInterval(id: i32);
///
///     #[wasm_bindgen(js_namespace = console)]
///     fn log(s: &str);
/// }
///
/// #[wasm_bindgen]
/// pub struct IntervalHandle {
///     interval_id: i32,
///     _closure: Closure<dyn FnMut()>,
/// }
///
/// impl Drop for IntervalHandle {
///     fn drop(&mut self) {
///         clearInterval(self.interval_id);
///     }
/// }
///
/// #[wasm_bindgen]
/// pub fn run() -> IntervalHandle {
///     // First up we use `Closure::new` to wrap up a Rust closure and create
///     // a JS closure.
///     let cb = Closure::new(|| {
///         log("interval elapsed!");
///     });
///
///     // Next we pass this via reference to the `setInterval` function, and
///     // `setInterval` gets a handle to the corresponding JS closure.
///     let interval_id = setInterval(&cb, 1_000);
///
///     // If we were to drop `cb` here it would cause an exception to be raised
///     // whenever the interval elapses. Instead we *return* our handle back to JS
///     // so JS can decide when to cancel the interval and deallocate the closure.
///     IntervalHandle {
///         interval_id,
///         _closure: cb,
///     }
/// }
/// ```
///
/// ## Casting a `Closure` to a `js_sys::Function`
///
/// This is the same `setInterval` example as above, except it is using
/// `web_sys` (which uses `js_sys::Function` for callbacks) instead of manually
/// writing bindings to `setInterval` and other Web APIs.
///
/// ```rust,ignore
/// use wasm_bindgen::JsCast;
///
/// #[wasm_bindgen]
/// pub struct IntervalHandle {
///     interval_id: i32,
///     _closure: Closure<dyn FnMut()>,
/// }
///
/// impl Drop for IntervalHandle {
///     fn drop(&mut self) {
///         let window = web_sys::window().unwrap();
///         window.clear_interval_with_handle(self.interval_id);
///     }
/// }
///
/// #[wasm_bindgen]
/// pub fn run() -> Result<IntervalHandle, JsValue> {
///     let cb = Closure::new(|| {
///         web_sys::console::log_1(&"interval elapsed!".into());
///     });
///
///     let window = web_sys::window().unwrap();
///     let interval_id = window.set_interval_with_callback_and_timeout_and_arguments_0(
///         // Note this method call, which uses `as_ref()` to get a `JsValue`
///         // from our `Closure` which is then converted to a `&Function`
///         // using the `JsCast::unchecked_ref` function.
///         cb.as_ref().unchecked_ref(),
///         1_000,
///     )?;
///
///     // Same as above.
///     Ok(IntervalHandle {
///         interval_id,
///         _closure: cb,
///     })
/// }
/// ```
///
/// ## Using `FnOnce` and `Closure::once` with `requestAnimationFrame`
///
/// Because `requestAnimationFrame` only calls its callback once, we can use
/// `FnOnce` and `Closure::once` with it.
///
/// ```rust,no_run
/// use wasm_bindgen::prelude::*;
///
/// #[wasm_bindgen]
/// extern "C" {
///     fn requestAnimationFrame(closure: &Closure<dyn FnMut()>) -> u32;
///     fn cancelAnimationFrame(id: u32);
///
///     #[wasm_bindgen(js_namespace = console)]
///     fn log(s: &str);
/// }
///
/// #[wasm_bindgen]
/// pub struct AnimationFrameHandle {
///     animation_id: u32,
///     _closure: Closure<dyn FnMut()>,
/// }
///
/// impl Drop for AnimationFrameHandle {
///     fn drop(&mut self) {
///         cancelAnimationFrame(self.animation_id);
///     }
/// }
///
/// // A type that will log a message when it is dropped.
/// struct LogOnDrop(&'static str);
/// impl Drop for LogOnDrop {
///     fn drop(&mut self) {
///         log(self.0);
///     }
/// }
///
/// #[wasm_bindgen]
/// pub fn run() -> AnimationFrameHandle {
///     // We are using `Closure::once` which takes a `FnOnce`, so the function
///     // can drop and/or move things that it closes over.
///     let fired = LogOnDrop("animation frame fired or canceled");
///     let cb = Closure::once(move || drop(fired));
///
///     // Schedule the animation frame!
///     let animation_id = requestAnimationFrame(&cb);
///
///     // Again, return a handle to JS, so that the closure is not dropped
///     // immediately and JS can decide whether to cancel the animation frame.
///     AnimationFrameHandle {
///         animation_id,
///         _closure: cb,
///     }
/// }
/// ```
///
/// ## Converting `FnOnce`s directly into JavaScript Functions with `Closure::once_into_js`
///
/// If we don't want to allow a `FnOnce` to be eagerly dropped (maybe because we
/// just want it to drop after it is called and don't care about cancellation)
/// then we can use the `Closure::once_into_js` function.
///
/// This is the same `requestAnimationFrame` example as above, but without
/// supporting early cancellation.
///
/// ```
/// use wasm_bindgen::prelude::*;
///
/// #[wasm_bindgen]
/// extern "C" {
///     // We modify the binding to take an untyped `JsValue` since that is what
///     // is returned by `Closure::once_into_js`.
///     //
///     // If we were using the `web_sys` binding for `requestAnimationFrame`,
///     // then the call sites would cast the `JsValue` into a `&js_sys::Function`
///     // using `f.unchecked_ref::<js_sys::Function>()`. See the `web_sys`
///     // example above for details.
///     fn requestAnimationFrame(callback: JsValue);
///
///     #[wasm_bindgen(js_namespace = console)]
///     fn log(s: &str);
/// }
///
/// // A type that will log a message when it is dropped.
/// struct LogOnDrop(&'static str);
/// impl Drop for LogOnDrop {
///     fn drop(&mut self) {
///         log(self.0);
///     }
/// }
///
/// #[wasm_bindgen]
/// pub fn run() {
///     // We are using `Closure::once_into_js` which takes a `FnOnce` and
///     // converts it into a JavaScript function, which is returned as a
///     // `JsValue`.
///     let fired = LogOnDrop("animation frame fired");
///     let cb = Closure::once_into_js(move || drop(fired));
///
///     // Schedule the animation frame!
///     requestAnimationFrame(cb);
///
///     // No need to worry about whether or not we drop a `Closure`
///     // here or return some sort of handle to JS!
/// }
/// ```
pub struct Closure<T: ?Sized> {
    js: ManuallyDrop<JsValue>,
    data: ManuallyDrop<Box<T>>,
}

union FatPtr<T: ?Sized> {
    ptr: *mut T,
    fields: (usize, usize),
}

impl<T> Closure<T>
where
    T: ?Sized + WasmClosure,
{
    /// Creates a new instance of `Closure` from the provided Rust function.
    ///
    /// Note that the closure provided here, `F`, has a few requirements
    /// associated with it:
    ///
    /// * It must implement `Fn` or `FnMut` (for `FnOnce` functions see
    ///   `Closure::once` and `Closure::once_into_js`).
    ///
    /// * It must be `'static`, aka no stack references (use the `move`
    ///   keyword).
    ///
    /// * It can have at most 7 arguments.
    ///
    /// * Its arguments and return values are all types that can be shared with
    ///   JS (i.e. have `#[wasm_bindgen]` annotations or are simple numbers,
    ///   etc.)
    pub fn new<F>(t: F) -> Closure<T>
    where
        F: IntoWasmClosure<T> + 'static,
    {
        Closure::wrap(Box::new(t).unsize())
    }

    /// A more direct version of `Closure::new` which creates a `Closure` from
    /// a `Box<dyn Fn>`/`Box<dyn FnMut>`, which is how it's kept internally.
    pub fn wrap(mut data: Box<T>) -> Closure<T> {
        assert_eq!(mem::size_of::<*const T>(), mem::size_of::<FatPtr<T>>());
        let (a, b) = unsafe {
            FatPtr {
                ptr: &mut *data as *mut T,
            }
            .fields
        };

        // Here we need to create a `JsValue` with the data and `T::invoke()`
        // function pointer. To do that we... take a few unconventional turns.
        // In essence what happens here is this:
        //
        // 1. First up, below we call a function, `breaks_if_inlined`. This
        //    function, as the name implies, does not work if it's inlined.
        //    More on that in a moment.
        // 2. This function internally calls a special import recognized by the
        //    `wasm-bindgen` CLI tool, `__wbindgen_describe_closure`. This
        //    imported symbol is similar to `__wbindgen_describe` in that it's
        //    not intended to show up in the final binary but it's an
        //    intermediate state for a `wasm-bindgen` binary.
        // 3. The `__wbindgen_describe_closure` import is namely passed a
        //    descriptor function, monomorphized for each invocation.
        //
        // Most of this doesn't actually make sense to happen at runtime! The
        // real magic happens when `wasm-bindgen` comes along and updates our
        // generated code. When `wasm-bindgen` runs it performs a few tasks:
        //
        // * First, it finds all functions that call
        //   `__wbindgen_describe_closure`. These are all `breaks_if_inlined`
        //   defined below as the symbol isn't called anywhere else.
        // * Next, `wasm-bindgen` executes the `breaks_if_inlined`
        //   monomorphized functions, passing it dummy arguments. This will
        //   execute the function just enough to invoke the special import,
        //   namely telling us about the function pointer that is the describe
        //   shim.
        // * This knowledge is then used to actually find the descriptor in the
        //   function table which is then executed to figure out the signature
        //   of the closure.
        // * Finally, and probably most heinously, the call to
        //   `breaks_if_inlined` is rewritten to call an otherwise globally
        //   imported function. This globally imported function will generate
        //   the `JsValue` for this closure specialized for the signature in
        //   question.
        //
        // Later on `wasm-gc` will clean up all the dead code and ensure that
        // we don't actually call `__wbindgen_describe_closure` at runtime. This
        // means we will end up not actually calling `breaks_if_inlined` in the
        // final binary, all calls to that function should be pruned.
        //
        // See crates/cli-support/src/js/closures.rs for a more information
        // about what's going on here.

        extern "C" fn describe<T: WasmClosure + ?Sized>() {
            inform(CLOSURE);
            T::describe()
        }

        #[inline(never)]
        unsafe fn breaks_if_inlined<T: WasmClosure + ?Sized>(a: usize, b: usize) -> u32 {
            super::__wbindgen_describe_closure(a as u32, b as u32, describe::<T> as u32)
        }

        let idx = unsafe { breaks_if_inlined::<T>(a, b) };

        Closure {
            js: ManuallyDrop::new(JsValue::_new(idx)),
            data: ManuallyDrop::new(data),
        }
    }

    /// Release memory management of this closure from Rust to the JS GC.
    ///
    /// When a `Closure` is dropped it will release the Rust memory and
    /// invalidate the associated JS closure, but this isn't always desired.
    /// Some callbacks are alive for the entire duration of the program or for a
    /// lifetime dynamically managed by the JS GC. This function can be used
    /// to drop this `Closure` while keeping the associated JS function still
    /// valid.
    ///
    /// By default this function will leak memory. This can be dangerous if this
    /// function is called many times in an application because the memory leak
    /// will overwhelm the page quickly and crash the wasm.
    ///
    /// If the browser, however, supports weak references, then this function
    /// will not leak memory. Instead the Rust memory will be reclaimed when the
    /// JS closure is GC'd. Weak references are not enabled by default since
    /// they're still a proposal for the JS standard. They can be enabled with
    /// `WASM_BINDGEN_WEAKREF=1` when running `wasm-bindgen`, however.
    pub fn into_js_value(self) -> JsValue {
        let idx = self.js.idx;
        mem::forget(self);
        JsValue::_new(idx)
    }

    /// Same as `into_js_value`, but doesn't return a value.
    pub fn forget(self) {
        drop(self.into_js_value());
    }
}

// NB: we use a specific `T` for this `Closure<T>` impl block to avoid every
// call site having to provide an explicit, turbo-fished type like
// `Closure::<dyn FnOnce()>::once(...)`.
impl Closure<dyn FnOnce()> {
    /// Create a `Closure` from a function that can only be called once.
    ///
    /// Since we have no way of enforcing that JS cannot attempt to call this
    /// `FnOne(A...) -> R` more than once, this produces a `Closure<dyn FnMut(A...)
    /// -> R>` that will dynamically throw a JavaScript error if called more
    /// than once.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wasm_bindgen::prelude::*;
    ///
    /// // Create an non-`Copy`, owned `String`.
    /// let mut s = String::from("Hello");
    ///
    /// // Close over `s`. Since `f` returns `s`, it is `FnOnce` and can only be
    /// // called once. If it was called a second time, it wouldn't have any `s`
    /// // to work with anymore!
    /// let f = move || {
    ///     s += ", World!";
    ///     s
    /// };
    ///
    /// // Create a `Closure` from `f`. Note that the `Closure`'s type parameter
    /// // is `FnMut`, even though `f` is `FnOnce`.
    /// let closure: Closure<dyn FnMut() -> String> = Closure::once(f);
    /// ```
    pub fn once<F, A, R>(fn_once: F) -> Closure<F::FnMut>
    where
        F: 'static + WasmClosureFnOnce<A, R>,
    {
        Closure::wrap(fn_once.into_fn_mut())
    }

    /// Convert a `FnOnce(A...) -> R` into a JavaScript `Function` object.
    ///
    /// If the JavaScript function is invoked more than once, it will throw an
    /// exception.
    ///
    /// Unlike `Closure::once`, this does *not* return a `Closure` that can be
    /// dropped before the function is invoked to deallocate the closure. The
    /// only way the `FnOnce` is deallocated is by calling the JavaScript
    /// function. If the JavaScript function is never called then the `FnOnce`
    /// and everything it closes over will leak.
    ///
    /// ```rust,ignore
    /// use wasm_bindgen::{prelude::*, JsCast};
    ///
    /// let f = Closure::once_into_js(move || {
    ///     // ...
    /// });
    ///
    /// assert!(f.is_instance_of::<js_sys::Function>());
    /// ```
    pub fn once_into_js<F, A, R>(fn_once: F) -> JsValue
    where
        F: 'static + WasmClosureFnOnce<A, R>,
    {
        fn_once.into_js_function()
    }
}

/// A trait for converting an `FnOnce(A...) -> R` into a `FnMut(A...) -> R` that
/// will throw if ever called more than once.
#[doc(hidden)]
pub trait WasmClosureFnOnce<A, R>: 'static {
    type FnMut: ?Sized + 'static + WasmClosure;

    fn into_fn_mut(self) -> Box<Self::FnMut>;

    fn into_js_function(self) -> JsValue;
}

impl<T: ?Sized> AsRef<JsValue> for Closure<T> {
    fn as_ref(&self) -> &JsValue {
        &self.js
    }
}

impl<T> WasmDescribe for Closure<T>
where
    T: WasmClosure + ?Sized,
{
    fn describe() {
        inform(EXTERNREF);
    }
}

// `Closure` can only be passed by reference to imports.
impl<'a, T> IntoWasmAbi for &'a Closure<T>
where
    T: WasmClosure + ?Sized,
{
    type Abi = u32;

    fn into_abi(self) -> u32 {
        (&*self.js).into_abi()
    }
}

impl<'a, T> OptionIntoWasmAbi for &'a Closure<T>
where
    T: WasmClosure + ?Sized,
{
    fn none() -> Self::Abi {
        0
    }
}

fn _check() {
    fn _assert<T: IntoWasmAbi>() {}
    _assert::<&Closure<dyn Fn()>>();
    _assert::<&Closure<dyn Fn(String)>>();
    _assert::<&Closure<dyn Fn() -> String>>();
    _assert::<&Closure<dyn FnMut()>>();
    _assert::<&Closure<dyn FnMut(String)>>();
    _assert::<&Closure<dyn FnMut() -> String>>();
}

impl<T> fmt::Debug for Closure<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Closure {{ ... }}")
    }
}

impl<T> Drop for Closure<T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        unsafe {
            // this will implicitly drop our strong reference in addition to
            // invalidating all future invocations of the closure
            if super::__wbindgen_cb_drop(self.js.idx) != 0 {
                ManuallyDrop::drop(&mut self.data);
            }
        }
    }
}

/// An internal trait for the `Closure` type.
///
/// This trait is not stable and it's not recommended to use this in bounds or
/// implement yourself.
#[doc(hidden)]
pub unsafe trait WasmClosure {
    fn describe();
}

/// An internal trait for the `Closure` type.
///
/// This trait is not stable and it's not recommended to use this in bounds or
/// implement yourself.
#[doc(hidden)]
pub trait IntoWasmClosure<T: ?Sized> {
    fn unsize(self: Box<Self>) -> Box<T>;
}

// The memory safety here in these implementations below is a bit tricky. We
// want to be able to drop the `Closure` object from within the invocation of a
// `Closure` for cases like promises. That means that while it's running we
// might drop the `Closure`, but that shouldn't invalidate the environment yet.
//
// Instead what we do is to wrap closures in `Rc` variables. The main `Closure`
// has a strong reference count which keeps the trait object alive. Each
// invocation of a closure then *also* clones this and gets a new reference
// count. When the closure returns it will release the reference count.
//
// This means that if the main `Closure` is dropped while it's being invoked
// then destruction is deferred until execution returns. Otherwise it'll
// deallocate data immediately.

macro_rules! doit {
    ($(
        ($($var:ident)*)
    )*) => ($(
        unsafe impl<$($var,)* R> WasmClosure for dyn Fn($($var),*) -> R + 'static
            where $($var: FromWasmAbi + 'static,)*
                  R: ReturnWasmAbi + 'static,
        {
            fn describe() {
                #[allow(non_snake_case)]
                unsafe extern "C" fn invoke<$($var: FromWasmAbi,)* R: ReturnWasmAbi>(
                    a: usize,
                    b: usize,
                    $($var: <$var as FromWasmAbi>::Abi),*
                ) -> <R as ReturnWasmAbi>::Abi {
                    if a == 0 {
                        throw_str("closure invoked after being dropped");
                    }
                    // Make sure all stack variables are converted before we
                    // convert `ret` as it may throw (for `Result`, for
                    // example)
                    let ret = {
                        let f: *const dyn Fn($($var),*) -> R =
                            FatPtr { fields: (a, b) }.ptr;
                        $(
                            let $var = <$var as FromWasmAbi>::from_abi($var);
                        )*
                        (*f)($($var),*)
                    };
                    ret.return_abi()
                }

                inform(invoke::<$($var,)* R> as u32);

                unsafe extern fn destroy<$($var: FromWasmAbi,)* R: ReturnWasmAbi>(
                    a: usize,
                    b: usize,
                ) {
                    // This can be called by the JS glue in erroneous situations
                    // such as when the closure has already been destroyed. If
                    // that's the case let's not make things worse by
                    // segfaulting and/or asserting, so just ignore null
                    // pointers.
                    if a == 0 {
                        return;
                    }
                    drop(Box::from_raw(FatPtr::<dyn Fn($($var,)*) -> R> {
                        fields: (a, b)
                    }.ptr));
                }
                inform(destroy::<$($var,)* R> as u32);

                <&Self>::describe();
            }
        }

        unsafe impl<$($var,)* R> WasmClosure for dyn FnMut($($var),*) -> R + 'static
            where $($var: FromWasmAbi + 'static,)*
                  R: ReturnWasmAbi + 'static,
        {
            fn describe() {
                #[allow(non_snake_case)]
                unsafe extern "C" fn invoke<$($var: FromWasmAbi,)* R: ReturnWasmAbi>(
                    a: usize,
                    b: usize,
                    $($var: <$var as FromWasmAbi>::Abi),*
                ) -> <R as ReturnWasmAbi>::Abi {
                    if a == 0 {
                        throw_str("closure invoked recursively or after being dropped");
                    }
                    // Make sure all stack variables are converted before we
                    // convert `ret` as it may throw (for `Result`, for
                    // example)
                    let ret = {
                        let f: *const dyn FnMut($($var),*) -> R =
                            FatPtr { fields: (a, b) }.ptr;
                        let f = f as *mut dyn FnMut($($var),*) -> R;
                        $(
                            let $var = <$var as FromWasmAbi>::from_abi($var);
                        )*
                        (*f)($($var),*)
                    };
                    ret.return_abi()
                }

                inform(invoke::<$($var,)* R> as u32);

                unsafe extern fn destroy<$($var: FromWasmAbi,)* R: ReturnWasmAbi>(
                    a: usize,
                    b: usize,
                ) {
                    // See `Fn()` above for why we simply return
                    if a == 0 {
                        return;
                    }
                    drop(Box::from_raw(FatPtr::<dyn FnMut($($var,)*) -> R> {
                        fields: (a, b)
                    }.ptr));
                }
                inform(destroy::<$($var,)* R> as u32);

                <&mut Self>::describe();
            }
        }

        #[allow(non_snake_case, unused_parens)]
        impl<T, $($var,)* R> WasmClosureFnOnce<($($var),*), R> for T
            where T: 'static + FnOnce($($var),*) -> R,
                  $($var: FromWasmAbi + 'static,)*
                  R: ReturnWasmAbi + 'static
        {
            type FnMut = dyn FnMut($($var),*) -> R;

            fn into_fn_mut(self) -> Box<Self::FnMut> {
                let mut me = Some(self);
                Box::new(move |$($var),*| {
                    let me = me.take().expect_throw("FnOnce called more than once");
                    me($($var),*)
                })
            }

            fn into_js_function(self) -> JsValue {
                use std::rc::Rc;
                use crate::__rt::WasmRefCell;

                let mut me = Some(self);

                let rc1 = Rc::new(WasmRefCell::new(None));
                let rc2 = rc1.clone();

                let closure = Closure::wrap(Box::new(move |$($var),*| {
                    // Invoke ourself and get the result.
                    let me = me.take().expect_throw("FnOnce called more than once");
                    let result = me($($var),*);

                    // And then drop the `Rc` holding this function's `Closure`
                    // alive.
                    debug_assert_eq!(Rc::strong_count(&rc2), 1);
                    let option_closure = rc2.borrow_mut().take();
                    debug_assert!(option_closure.is_some());
                    drop(option_closure);

                    result
                }) as Box<dyn FnMut($($var),*) -> R>);

                let js_val = closure.as_ref().clone();

                *rc1.borrow_mut() = Some(closure);
                debug_assert_eq!(Rc::strong_count(&rc1), 2);
                drop(rc1);

                js_val
            }
        }

        impl<T, $($var,)* R> IntoWasmClosure<dyn FnMut($($var),*) -> R> for T
            where T: 'static + FnMut($($var),*) -> R,
                  $($var: FromWasmAbi + 'static,)*
                  R: ReturnWasmAbi + 'static,
        {
            fn unsize(self: Box<Self>) -> Box<dyn FnMut($($var),*) -> R> { self }
        }

        impl<T, $($var,)* R> IntoWasmClosure<dyn Fn($($var),*) -> R> for T
            where T: 'static + Fn($($var),*) -> R,
                  $($var: FromWasmAbi + 'static,)*
                  R: ReturnWasmAbi + 'static,
        {
            fn unsize(self: Box<Self>) -> Box<dyn Fn($($var),*) -> R> { self }
        }
    )*)
}

doit! {
    ()
    (A)
    (A B)
    (A B C)
    (A B C D)
    (A B C D E)
    (A B C D E F)
    (A B C D E F G)
    (A B C D E F G H)
}

// Copy the above impls down here for where there's only one argument and it's a
// reference. We could add more impls for more kinds of references, but it
// becomes a combinatorial explosion quickly. Let's see how far we can get with
// just this one! Maybe someone else can figure out voodoo so we don't have to
// duplicate.

unsafe impl<A, R> WasmClosure for dyn Fn(&A) -> R
where
    A: RefFromWasmAbi,
    R: ReturnWasmAbi + 'static,
{
    fn describe() {
        #[allow(non_snake_case)]
        unsafe extern "C" fn invoke<A: RefFromWasmAbi, R: ReturnWasmAbi>(
            a: usize,
            b: usize,
            arg: <A as RefFromWasmAbi>::Abi,
        ) -> <R as ReturnWasmAbi>::Abi {
            if a == 0 {
                throw_str("closure invoked after being dropped");
            }
            // Make sure all stack variables are converted before we
            // convert `ret` as it may throw (for `Result`, for
            // example)
            let ret = {
                let f: *const dyn Fn(&A) -> R = FatPtr { fields: (a, b) }.ptr;
                let arg = <A as RefFromWasmAbi>::ref_from_abi(arg);
                (*f)(&*arg)
            };
            ret.return_abi()
        }

        inform(invoke::<A, R> as u32);

        unsafe extern "C" fn destroy<A: RefFromWasmAbi, R: ReturnWasmAbi>(a: usize, b: usize) {
            // See `Fn()` above for why we simply return
            if a == 0 {
                return;
            }
            drop(Box::from_raw(
                FatPtr::<dyn Fn(&A) -> R> { fields: (a, b) }.ptr,
            ));
        }
        inform(destroy::<A, R> as u32);

        <&Self>::describe();
    }
}

unsafe impl<A, R> WasmClosure for dyn FnMut(&A) -> R
where
    A: RefFromWasmAbi,
    R: ReturnWasmAbi + 'static,
{
    fn describe() {
        #[allow(non_snake_case)]
        unsafe extern "C" fn invoke<A: RefFromWasmAbi, R: ReturnWasmAbi>(
            a: usize,
            b: usize,
            arg: <A as RefFromWasmAbi>::Abi,
        ) -> <R as ReturnWasmAbi>::Abi {
            if a == 0 {
                throw_str("closure invoked recursively or after being dropped");
            }
            // Make sure all stack variables are converted before we
            // convert `ret` as it may throw (for `Result`, for
            // example)
            let ret = {
                let f: *const dyn FnMut(&A) -> R = FatPtr { fields: (a, b) }.ptr;
                let f = f as *mut dyn FnMut(&A) -> R;
                let arg = <A as RefFromWasmAbi>::ref_from_abi(arg);
                (*f)(&*arg)
            };
            ret.return_abi()
        }

        inform(invoke::<A, R> as u32);

        unsafe extern "C" fn destroy<A: RefFromWasmAbi, R: ReturnWasmAbi>(a: usize, b: usize) {
            // See `Fn()` above for why we simply return
            if a == 0 {
                return;
            }
            drop(Box::from_raw(
                FatPtr::<dyn FnMut(&A) -> R> { fields: (a, b) }.ptr,
            ));
        }
        inform(destroy::<A, R> as u32);

        <&mut Self>::describe();
    }
}

#[allow(non_snake_case)]
impl<T, A, R> WasmClosureFnOnce<(&A,), R> for T
where
    T: 'static + FnOnce(&A) -> R,
    A: RefFromWasmAbi + 'static,
    R: ReturnWasmAbi + 'static,
{
    type FnMut = dyn FnMut(&A) -> R;

    fn into_fn_mut(self) -> Box<Self::FnMut> {
        let mut me = Some(self);
        Box::new(move |arg| {
            let me = me.take().expect_throw("FnOnce called more than once");
            me(arg)
        })
    }

    fn into_js_function(self) -> JsValue {
        use crate::__rt::WasmRefCell;
        use std::rc::Rc;

        let mut me = Some(self);

        let rc1 = Rc::new(WasmRefCell::new(None));
        let rc2 = rc1.clone();

        let closure = Closure::wrap(Box::new(move |arg: &A| {
            // Invoke ourself and get the result.
            let me = me.take().expect_throw("FnOnce called more than once");
            let result = me(arg);

            // And then drop the `Rc` holding this function's `Closure`
            // alive.
            debug_assert_eq!(Rc::strong_count(&rc2), 1);
            let option_closure = rc2.borrow_mut().take();
            debug_assert!(option_closure.is_some());
            drop(option_closure);

            result
        }) as Box<dyn FnMut(&A) -> R>);

        let js_val = closure.as_ref().clone();

        *rc1.borrow_mut() = Some(closure);
        debug_assert_eq!(Rc::strong_count(&rc1), 2);
        drop(rc1);

        js_val
    }
}
