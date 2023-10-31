extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct A {
    x: u32,
}

#[wasm_bindgen]
impl A {
    pub fn new() -> A {
        A { x: 3 }
    }

    pub fn foo(&self) {
        assert_eq!(self.x, 3);
    }
}

#[wasm_bindgen]
pub fn foo(x: bool) {
    A::new().foo();

    if x {
        bar("test");
        baz(JsValue::from(3));
    }
}

#[wasm_bindgen]
extern "C" {
    fn some_import();
    static A: JsValue;
}

#[wasm_bindgen]
pub fn bar(_: &str) -> JsValue {
    some_import();
    A.clone()
}

#[wasm_bindgen]
pub fn baz(_: JsValue) {}

#[test]
fn test_foo() {
    foo(false);
    A::new().foo();
}
