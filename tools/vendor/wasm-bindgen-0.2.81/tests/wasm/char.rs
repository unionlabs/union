use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/char.js")]
extern "C" {
    fn js_identity(c: char) -> char;
    fn js_works();
}

#[wasm_bindgen]
pub fn rust_identity(c: char) -> char {
    c
}

#[wasm_bindgen]
pub fn rust_js_identity(c: char) -> char {
    js_identity(c)
}

#[wasm_bindgen]
pub fn letter() -> char {
    'a'
}

#[wasm_bindgen]
pub fn face() -> char {
    '😀'
}

#[wasm_bindgen]
pub fn rust_letter(a: char) {
    assert_eq!(a, 'a');
}

#[wasm_bindgen]
pub fn rust_face(p: char) {
    assert_eq!(p, '😀');
}

#[wasm_bindgen_test]
fn works() {
    js_works();
}
