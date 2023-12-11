use rusty_jsc::JSValue;

use crate::jsc::trap::Trap;
use crate::RuntimeError;

impl From<Trap> for RuntimeError {
    fn from(trap: Trap) -> Self {
        if trap.is::<RuntimeError>() {
            return trap.downcast::<RuntimeError>().unwrap();
        }
        let wasm_trace = vec![];
        let trap_code = None;
        // let (wasm_trace, trap_code) = wasmer_compiler::get_trace_and_trapcode(&trap);
        RuntimeError::new_from_source(trap, wasm_trace, trap_code)
    }
}

impl From<RuntimeError> for JSValue {
    fn from(_err: RuntimeError) -> Self {
        // err.inner.source.into()
        unimplemented!();
    }
}

pub(crate) fn raise(error: Box<dyn std::error::Error + Send + Sync>) -> ! {
    unimplemented!()
    // let error = Trap::user(error);
    // let js_error: JsValue = error.into();
    // wasm_bindgen::throw_val(js_error)
}
