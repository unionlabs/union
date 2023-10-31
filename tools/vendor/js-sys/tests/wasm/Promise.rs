use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn promise_inheritance() {
    let promise = Promise::new(&mut |_, _| ());
    assert!(promise.is_instance_of::<Promise>());
    assert!(promise.is_instance_of::<Object>());
    let _: &Object = promise.as_ref();
}
