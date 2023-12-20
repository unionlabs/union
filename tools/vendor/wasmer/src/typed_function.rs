//! Native Functions.
//!
//! This module creates the helper `TypedFunction` that let us call WebAssembly
//! functions with the native ABI, that is:
//!
//! ```ignore
//! let add_one = instance.exports.get_function("function_name")?;
//! let add_one_native: TypedFunction<i32, i32> = add_one.native().unwrap();
//! ```
use crate::{Function, WasmTypeList};
use std::marker::PhantomData;

use crate::store::AsStoreRef;

/// A WebAssembly function that can be called natively
/// (using the Native ABI).
#[derive(Clone)]
pub struct TypedFunction<Args, Rets> {
    pub(crate) func: Function,
    _phantom: PhantomData<fn(Args) -> Rets>,
}

unsafe impl<Args, Rets> Send for TypedFunction<Args, Rets> {}
unsafe impl<Args, Rets> Sync for TypedFunction<Args, Rets> {}

impl<Args, Rets> TypedFunction<Args, Rets>
where
    Args: WasmTypeList,
    Rets: WasmTypeList,
{
    #[allow(dead_code)]
    pub(crate) fn new(_store: &impl AsStoreRef, func: Function) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn into_function(self) -> Function {
        self.func
    }
}
