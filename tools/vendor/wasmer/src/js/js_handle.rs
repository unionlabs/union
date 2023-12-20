use std::ops::{Deref, DerefMut};

use wasm_bindgen::JsValue;

use self::integrity_check::IntegrityCheck;

/// A handle that lets you detect thread-safety issues when passing a
/// [`JsValue`] (or derived type) around.
#[derive(Debug, Clone)]
pub(crate) struct JsHandle<T> {
    value: T,
    integrity: IntegrityCheck,
}

impl<T> JsHandle<T> {
    #[track_caller]
    pub fn new(value: T) -> Self {
        JsHandle {
            value,
            integrity: IntegrityCheck::new(std::any::type_name::<T>()),
        }
    }

    #[track_caller]
    pub fn into_inner(self) -> T {
        self.integrity.check();
        self.value
    }
}

impl<T: PartialEq> PartialEq for JsHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        let JsHandle {
            value,
            integrity: _,
        } = self;

        *value == other.value
    }
}

impl<T: Eq> Eq for JsHandle<T> {}

impl<T> From<T> for JsHandle<T> {
    #[track_caller]
    fn from(value: T) -> Self {
        JsHandle::new(value)
    }
}

impl<T: Into<JsValue>> From<JsHandle<T>> for JsValue {
    fn from(value: JsHandle<T>) -> Self {
        value.into_inner().into()
    }
}

impl<A, T> AsRef<A> for JsHandle<T>
where
    T: AsRef<A>,
{
    #[track_caller]
    fn as_ref(&self) -> &A {
        self.integrity.check();
        self.value.as_ref()
    }
}

impl<T> Deref for JsHandle<T> {
    type Target = T;

    #[track_caller]
    fn deref(&self) -> &Self::Target {
        self.integrity.check();
        &self.value
    }
}

impl<T> DerefMut for JsHandle<T> {
    #[track_caller]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.integrity.check();
        &mut self.value
    }
}

#[cfg(not(debug_assertions))]
mod integrity_check {

    #[derive(Debug, Clone, PartialEq)]
    pub(crate) struct IntegrityCheck;

    impl IntegrityCheck {
        #[track_caller]
        pub(crate) fn new(_type_name: &'static str) -> Self {
            IntegrityCheck
        }

        pub(crate) fn check(&self) {}
    }
}

#[cfg(debug_assertions)]
mod integrity_check {
    use std::{
        fmt::Write as _,
        panic::Location,
        sync::atomic::{AtomicU32, Ordering},
    };

    use js_sys::{JsString, Symbol};
    use wasm_bindgen::JsValue;

    #[derive(Debug, Clone, PartialEq)]
    pub(crate) struct IntegrityCheck {
        original_thread: u32,
        created: &'static Location<'static>,
        type_name: &'static str,
        backtrace: Option<String>,
    }

    impl IntegrityCheck {
        #[track_caller]
        pub(crate) fn new(type_name: &'static str) -> Self {
            IntegrityCheck {
                original_thread: current_thread_id(),
                created: Location::caller(),
                type_name,
                backtrace: record_backtrace(),
            }
        }

        #[track_caller]
        pub(crate) fn check(&self) {
            let current_thread = current_thread_id();

            if current_thread != self.original_thread {
                let IntegrityCheck {
                    original_thread,
                    created,
                    type_name,
                    backtrace,
                } = self;
                let mut error_message = String::new();

                writeln!(
                    error_message,
                    "Thread-safety integrity check for {type_name} failed."
                )
                .unwrap();

                writeln!(
                    error_message,
                    "Created at {created} on thread #{original_thread}"
                )
                .unwrap();

                if let Some(bt) = backtrace {
                    writeln!(error_message, "{bt}").unwrap();
                    writeln!(error_message).unwrap();
                }

                let caller = Location::caller();

                writeln!(
                    error_message,
                    "Accessed from {caller} on thread #{current_thread}"
                )
                .unwrap();

                if let Some(bt) = record_backtrace() {
                    writeln!(error_message, "{bt}").unwrap();
                    writeln!(error_message).unwrap();
                }

                panic!("{error_message}");
            }
        }
    }

    /// Get a unique ID for the current "thread" (i.e. web worker or the main
    /// thread).
    ///
    /// This works by creating a `$WASMER_THREAD_ID` symbol and setting it on
    /// the global object.
    fn current_thread_id() -> u32 {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);

        let global = js_sys::global();
        let thread_id_symbol = Symbol::for_("$WASMER_THREAD_ID");

        if let Some(v) = js_sys::Reflect::get(&global, &thread_id_symbol)
            .ok()
            .and_then(|v| v.as_f64())
        {
            // Note: we use a symbol so we know for sure that nobody else created
            // this field.
            return v as u32;
        }

        // Looks like we haven't set the thread ID yet.
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

        js_sys::Reflect::set(&global, &thread_id_symbol, &JsValue::from(id))
            .expect("Setting a field on the global object should never fail");

        id
    }

    fn record_backtrace() -> Option<String> {
        let err = js_sys::Error::new("");
        let stack = JsString::from(wasm_bindgen::intern("stack"));

        js_sys::Reflect::get(&err, &stack)
            .ok()
            .and_then(|v| v.as_string())
    }
}
