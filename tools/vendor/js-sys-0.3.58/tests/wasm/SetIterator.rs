use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn entries() {
    let s = Set::new(&JsValue::undefined());
    s.add(&1.into());
    let iter = s.entries();
    let obj = iter.next().unwrap();
    assert!(!obj.done());
    let array = Array::from(&obj.value());
    assert_eq!(array.length(), 2);
    array.for_each(&mut |a, _, _| {
        assert_eq!(a, 1);
    });

    assert!(iter.next().unwrap().done());
}

#[wasm_bindgen_test]
fn keys() {
    let s = Set::new(&JsValue::undefined());
    s.add(&1.into());
    let iter = s.keys();
    let obj = iter.next().unwrap();
    assert!(!obj.done());
    assert_eq!(obj.value(), 1);
    assert!(iter.next().unwrap().done());
}

#[wasm_bindgen_test]
fn values() {
    let s = Set::new(&JsValue::undefined());
    s.add(&1.into());
    let iter = s.values();
    let obj = iter.next().unwrap();
    assert!(!obj.done());
    assert_eq!(obj.value(), 1);
    assert!(iter.next().unwrap().done());
}
