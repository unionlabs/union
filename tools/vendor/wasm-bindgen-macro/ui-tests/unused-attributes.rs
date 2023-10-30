#![deny(unused_variables)]

use wasm_bindgen::prelude::*;

struct A {}

#[wasm_bindgen]
impl A {
    #[wasm_bindgen(method)]
    pub fn foo() {}
}

#[wasm_bindgen]
pub struct MyStruct {
    hello: String,
}

#[wasm_bindgen(getter, typescript_custom_section)]
pub const FOO: &'static str = "FOO";

#[wasm_bindgen(readonly)]
pub fn bar() {}

#[wasm_bindgen(getter_with_clone, final)]
impl MyStruct {
    #[wasm_bindgen(getter, typescript_type = "Thing[]")]
    pub fn hello(&self) -> String {
        self.hello.clone()
    }
}

fn main() {}
