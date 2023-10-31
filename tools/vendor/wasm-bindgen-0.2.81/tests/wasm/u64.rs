use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/u64.js")]
extern "C" {
    fn i64_js_identity(a: i64) -> i64;
    fn u64_js_identity(a: u64) -> u64;
    fn js_works();
}

#[wasm_bindgen]
pub fn zero() -> u64 {
    0
}

#[wasm_bindgen]
pub fn one() -> u64 {
    1
}

#[wasm_bindgen]
pub fn neg_one() -> i64 {
    -1
}

#[wasm_bindgen]
pub fn i32_min() -> i64 {
    i32::min_value() as i64
}

#[wasm_bindgen]
pub fn u32_max() -> u64 {
    u32::max_value() as u64
}

#[wasm_bindgen]
pub fn i64_min() -> i64 {
    i64::min_value()
}

#[wasm_bindgen]
pub fn u64_max() -> u64 {
    u64::max_value()
}

#[wasm_bindgen]
pub fn i64_rust_identity(a: i64) -> i64 {
    i64_js_identity(a)
}

#[wasm_bindgen]
pub fn u64_rust_identity(a: u64) -> u64 {
    u64_js_identity(a)
}

#[wasm_bindgen]
pub fn i64_slice(a: &[i64]) -> Vec<i64> {
    a.to_vec()
}

#[wasm_bindgen]
pub fn u64_slice(a: &[u64]) -> Vec<u64> {
    a.to_vec()
}

#[wasm_bindgen_test]
fn works() {
    js_works();
}
