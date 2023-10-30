use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    type Foo;

    #[wasm_bindgen(method, structural, final)]
    fn bar(this: &Foo);
}

fn main() {}
