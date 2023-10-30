use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/arg_names.js")]
extern "C" {
    fn js_arg_names();
}

#[wasm_bindgen]
pub fn fn_with_many_args(_a: i32, _b: i32, _c: i32, _d: i32) {}

#[wasm_bindgen_test]
fn rust_arg_names() {
    js_arg_names();
}
