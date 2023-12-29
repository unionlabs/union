use rusty_jsc::{JSContext, JSObject, JSValue};

use crate::RuntimeError;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum InnerTrap {
    User(Box<dyn Error + Send + Sync>),
    JSC(JSValue),
}

/// A struct representing a Trap
#[derive(Debug)]
pub struct Trap {
    inner: InnerTrap,
}

unsafe impl Send for Trap {}
unsafe impl Sync for Trap {}

impl Trap {
    pub fn user(error: Box<dyn Error + Send + Sync>) -> Self {
        Self {
            inner: InnerTrap::User(error),
        }
    }

    /// Attempts to downcast the `Trap` to a concrete type.
    pub fn downcast<T: Error + 'static>(self) -> Result<T, Self> {
        match self.inner {
            // We only try to downcast user errors
            InnerTrap::User(err) if err.is::<T>() => Ok(*err.downcast::<T>().unwrap()),
            _ => Err(self),
        }
    }

    /// Attempts to downcast the `Trap` to a concrete type.
    pub fn downcast_ref<T: Error + 'static>(&self) -> Option<&T> {
        match &self.inner {
            // We only try to downcast user errors
            InnerTrap::User(err) if err.is::<T>() => err.downcast_ref::<T>(),
            _ => None,
        }
    }

    /// Returns true if the `Trap` is the same as T
    pub fn is<T: Error + 'static>(&self) -> bool {
        match &self.inner {
            InnerTrap::User(err) => err.is::<T>(),
            _ => false,
        }
    }

    pub(crate) fn into_jsvalue(self, ctx: &JSContext) -> JSValue {
        match self.inner {
            InnerTrap::User(err) => {
                let obj = JSObject::new(ctx);
                let err_ptr = Box::leak(Box::new(err));
                let wasmer_error_ptr = JSValue::number(&ctx, err_ptr as *mut _ as usize as _);
                obj.set_property(&ctx, "wasmer_error_ptr".to_string(), wasmer_error_ptr)
                    .unwrap();
                obj.to_jsvalue()
            }
            InnerTrap::JSC(value) => value,
        }
    }

    pub(crate) fn from_jsvalue(ctx: &JSContext, val: JSValue) -> Self {
        let obj_val = val.to_object(ctx).unwrap();
        let wasmer_error_ptr = obj_val.get_property(&ctx, "wasmer_error_ptr".to_string());
        if wasmer_error_ptr.is_number(ctx) {
            let err_ptr = wasmer_error_ptr.to_number(ctx).unwrap() as usize
                as *mut Box<dyn Error + Send + Sync>;
            let err = unsafe { Box::from_raw(err_ptr) };
            return Self::user(*err);
        }
        Self {
            inner: InnerTrap::JSC(val),
        }
    }
}

impl std::error::Error for Trap {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.inner {
            InnerTrap::User(err) => Some(&**err),
            _ => None,
        }
    }
}

impl fmt::Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            InnerTrap::User(e) => write!(f, "user: {}", e),
            InnerTrap::JSC(value) => write!(f, "jsc: obscure"),
        }
    }
}

impl From<JSValue> for RuntimeError {
    fn from(original: JSValue) -> Self {
        let trap = Trap {
            inner: InnerTrap::JSC(original),
        };
        trap.into()
        // unimplemented!("TODO: implement Trap::from(JSValue) for RuntimeError");
        // // We try to downcast the error and see if it's
        // // an instance of RuntimeError instead, so we don't need
        // // to re-wrap it.
        // let trap = Trap::downcast_js(original).unwrap_or_else(|o| Trap {
        //     inner: InnerTrap::JSC(o),
        // });
        // trap.into()
    }
}
