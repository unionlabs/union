use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use wasm_bindgen_test_crate_a as a;
use wasm_bindgen_test_crate_b as b;

#[wasm_bindgen(module = "tests/wasm/duplicate_deps.js")]
extern "C" {
    fn assert_next_undefined();
    fn assert_next_ten();
}

#[wasm_bindgen_test]
fn works() {
    assert_next_undefined();
    a::test();
    assert_next_ten();
    b::test();
}
