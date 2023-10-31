use wasm_bindgen_test::*;
use web_sys::console;

#[wasm_bindgen_test]
fn test_console() {
    console::time_with_label("test label");
    console::time_end_with_label("test label");
}
