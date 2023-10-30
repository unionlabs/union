use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/usize.js")]
extern "C" {
    fn isize_js_identity(a: isize) -> isize;
    fn usize_js_identity(a: usize) -> usize;
    fn js_works();
}

#[wasm_bindgen]
pub fn usize_zero() -> usize {
    0
}

#[wasm_bindgen]
pub fn usize_one() -> usize {
    1
}

#[wasm_bindgen]
pub fn isize_neg_one() -> isize {
    -1
}

#[wasm_bindgen]
pub fn isize_i32_min() -> isize {
    i32::min_value() as isize
}

#[wasm_bindgen]
pub fn usize_u32_max() -> usize {
    u32::max_value() as usize
}

#[wasm_bindgen]
pub fn isize_min() -> isize {
    isize::min_value()
}

#[wasm_bindgen]
pub fn usize_max() -> usize {
    usize::max_value()
}

#[wasm_bindgen]
pub fn isize_rust_identity(a: isize) -> isize {
    isize_js_identity(a)
}

#[wasm_bindgen]
pub fn usize_rust_identity(a: usize) -> usize {
    usize_js_identity(a)
}

#[wasm_bindgen]
pub fn isize_slice(a: &[isize]) -> Vec<isize> {
    a.to_vec()
}

#[wasm_bindgen]
pub fn usize_slice(a: &[usize]) -> Vec<usize> {
    a.to_vec()
}

#[wasm_bindgen]
pub async fn async_usize_one() -> usize {
    1
}

#[wasm_bindgen]
pub async fn async_isize_neg_one() -> isize {
    -1
}

#[wasm_bindgen_test]
fn works() {
    js_works();
}
