// use super::frame_info::{FrameInfo, GlobalFrameInfo, FRAME_INFO};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

/// A struct representing an aborted instruction execution, with a message
/// indicating the cause.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmerRuntimeError {
    inner: Arc<RuntimeErrorSource>,
}

/// This type is the same as `WasmerRuntimeError`.
///
/// We use the `WasmerRuntimeError` name to not collide with the
/// `RuntimeError` in JS.
pub type RuntimeError = WasmerRuntimeError;

impl PartialEq for RuntimeError {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

/// The source of the `RuntimeError`.
#[derive(Debug)]
enum RuntimeErrorSource {
    Generic(String),
    User(Box<dyn Error + Send + Sync>),
    Js(JsValue),
}

/// This is a hack to ensure the error type is Send+Sync
unsafe impl Send for RuntimeErrorSource {}
unsafe impl Sync for RuntimeErrorSource {}

impl fmt::Display for RuntimeErrorSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic(s) => write!(f, "{}", s),
            Self::User(s) => write!(f, "{}", s),
            Self::Js(s) => write!(f, "{:?}", s),
        }
    }
}

impl RuntimeError {
    /// Creates a new generic `RuntimeError` with the given `message`.
    ///
    /// # Example
    /// ```
    /// let trap = wasmer_engine::RuntimeError::new("unexpected error");
    /// assert_eq!("unexpected error", trap.message());
    /// ```
    pub fn new<I: Into<String>>(message: I) -> Self {
        RuntimeError {
            inner: Arc::new(RuntimeErrorSource::Generic(message.into())),
        }
    }

    /// Raises a custom user Error
    #[deprecated(since = "2.1.1", note = "return a Result from host functions instead")]
    pub fn raise(error: Box<dyn Error + Send + Sync>) -> ! {
        let error = Self::user(error);
        let js_error: JsValue = error.into();
        wasm_bindgen::throw_val(js_error)
    }

    /// Creates a custom user Error.
    ///
    /// This error object can be passed through Wasm frames and later retrieved
    /// using the `downcast` method.
    pub fn user(error: Box<dyn Error + Send + Sync>) -> Self {
        match error.downcast::<Self>() {
            // The error is already a RuntimeError, we return it directly
            Ok(runtime_error) => *runtime_error,
            Err(error) => RuntimeError {
                inner: Arc::new(RuntimeErrorSource::User(error)),
            },
        }
    }

    /// Returns a reference the `message` stored in `Trap`.
    pub fn message(&self) -> String {
        format!("{}", self.inner)
    }

    /// Attempts to downcast the `RuntimeError` to a concrete type.
    pub fn downcast<T: Error + 'static>(self) -> Result<T, Self> {
        match Arc::try_unwrap(self.inner) {
            // We only try to downcast user errors
            Ok(RuntimeErrorSource::User(err)) if err.is::<T>() => Ok(*err.downcast::<T>().unwrap()),
            Ok(inner) => Err(Self {
                inner: Arc::new(inner),
            }),
            Err(inner) => Err(Self { inner }),
        }
    }

    /// Returns true if the `RuntimeError` is the same as T
    pub fn is<T: Error + 'static>(&self) -> bool {
        match self.inner.as_ref() {
            RuntimeErrorSource::User(err) => err.is::<T>(),
            _ => false,
        }
    }
}

impl fmt::Debug for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeError")
            .field("source", &self.inner)
            .finish()
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeError: {}", self.message())?;
        Ok(())
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.inner.as_ref() {
            RuntimeErrorSource::User(err) => Some(&**err),
            _ => None,
        }
    }
}

pub fn generic_of_jsval<T: FromWasmAbi<Abi = u32>>(
    js: JsValue,
    classname: &str,
) -> Result<T, JsValue> {
    use js_sys::{Object, Reflect};
    let ctor_name = Object::get_prototype_of(&js).constructor().name();
    if ctor_name == classname {
        let ptr = Reflect::get(&js, &JsValue::from_str("ptr"))?;
        match ptr.as_f64() {
            Some(ptr_f64) => {
                let foo = unsafe { T::from_abi(ptr_f64 as u32) };
                Ok(foo)
            }
            None => {
                // We simply relay the js value
                Err(js)
            }
        }
    } else {
        Err(js)
    }
}

impl From<JsValue> for RuntimeError {
    fn from(original: JsValue) -> Self {
        // We try to downcast the error and see if it's
        // an instance of RuntimeError instead, so we don't need
        // to re-wrap it.
        generic_of_jsval(original, "WasmerRuntimeError").unwrap_or_else(|js| RuntimeError {
            inner: Arc::new(RuntimeErrorSource::Js(js)),
        })
    }
}
