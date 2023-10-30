use wasm_bindgen::prelude::*;

#[wasm_bindgen]
fn foo() {}

#[wasm_bindgen]
pub const fn foo2() {}

#[wasm_bindgen]
struct Foo<T>(T);

#[wasm_bindgen]
extern "C" {
    static mut FOO: u32;

    pub fn foo3(x: i32, ...);
}

#[wasm_bindgen]
extern "system" {
}

#[wasm_bindgen]
pub fn foo4<T>() {}
#[wasm_bindgen]
pub fn foo5<'a>() {}
#[wasm_bindgen]
pub fn foo6<'a, T>() {}

#[wasm_bindgen]
trait X {}

fn main() {}
