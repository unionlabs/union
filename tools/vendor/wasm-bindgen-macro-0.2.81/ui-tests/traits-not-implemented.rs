use wasm_bindgen::prelude::*;

struct A;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn foo(a: A);
}

fn main() {}
