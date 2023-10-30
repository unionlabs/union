
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    type A;

    #[wasm_bindgen(setter, method)]
    fn a(this: &A, b: i32);

    #[wasm_bindgen(setter = x, method)]
    fn b(this: &A, b: i32);

    #[wasm_bindgen(setter, method, js_name = x)]
    fn c(this: &A, b: i32);
}

fn main() {}
