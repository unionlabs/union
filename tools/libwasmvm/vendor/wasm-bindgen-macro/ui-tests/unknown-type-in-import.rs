use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub fn foo(a: A);
}

fn main() {}

