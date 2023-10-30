use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::Performance;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = performance)]
    static PERFORMANCE: Performance;
}

#[wasm_bindgen_test]
fn to_json() {
    let perf = JsValue::from(PERFORMANCE.to_json());
    assert!(perf.is_object());
}
