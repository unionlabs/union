use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Foo;
}

#[wasm_bindgen]
struct Bar {
    pub a: Foo,
    #[wasm_bindgen(getter_with_clone)]
    pub b: Foo,
}

fn main() {}
