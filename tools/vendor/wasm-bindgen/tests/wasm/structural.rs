use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/structural.js")]
extern "C" {
    fn js_works();
}

#[wasm_bindgen]
extern "C" {
    pub type StructuralFoo;

    #[wasm_bindgen(method, structural)]
    fn bar(this: &StructuralFoo);
    #[wasm_bindgen(method, getter, structural)]
    fn baz(this: &StructuralFoo) -> u32;
    #[wasm_bindgen(method, setter, structural)]
    fn set_baz(this: &StructuralFoo, val: u32);
}

#[wasm_bindgen]
pub fn run(a: &StructuralFoo) {
    a.bar();
    assert_eq!(a.baz(), 1);
    a.set_baz(2);
    assert_eq!(a.baz(), 2);
}

#[wasm_bindgen_test]
fn works() {
    js_works();
}
