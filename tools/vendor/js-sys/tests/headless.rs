#![cfg(target_arch = "wasm32")]

extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_test;

use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/headless.js")]
extern "C" {
    fn is_array_values_supported() -> bool;
}

#[wasm_bindgen]
extern "C" {
    type ValuesIterator;
    #[wasm_bindgen(method, structural)]
    fn next(this: &ValuesIterator) -> IterNext;

    type IterNext;

    #[wasm_bindgen(method, getter, structural)]
    fn value(this: &IterNext) -> JsValue;
    #[wasm_bindgen(method, getter, structural)]
    fn done(this: &IterNext) -> bool;
}

#[wasm_bindgen_test]
fn array_iterator_values() {
    if !is_array_values_supported() {
        return;
    }
    let array = Array::new();
    array.push(&8.into());
    array.push(&3.into());
    array.push(&2.into());
    let iter = ValuesIterator::from(JsValue::from(array.values()));

    assert_eq!(iter.next().value(), 8);
    assert_eq!(iter.next().value(), 3);
    assert_eq!(iter.next().value(), 2);
    assert!(iter.next().done());
}
