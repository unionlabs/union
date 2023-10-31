use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/rethrow.js")]
extern "C" {
    fn call_throw_one();
    fn call_ok();
}

#[wasm_bindgen_test]
fn err_works() {
    call_throw_one();
}

#[wasm_bindgen]
pub fn throw_one() -> Result<u32, JsValue> {
    Err(1.into())
}

#[wasm_bindgen_test]
fn ok_works() {
    call_ok();
}

#[wasm_bindgen]
pub fn nothrow() -> Result<u32, JsValue> {
    Ok(1)
}
