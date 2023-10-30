use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(raw_module = "./tests/headless/modules.js")]
extern "C" {
    fn get_five() -> u32;
}

#[wasm_bindgen_test]
fn test_get_five() {
    assert_eq!(get_five(), 5);
}
