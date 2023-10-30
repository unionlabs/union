use js_sys::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn keys() {
    let array = Array::new();
    array.push(&JsValue::from(1));
    array.push(&JsValue::from(2));
    array.push(&JsValue::from(3));
    array.push(&JsValue::from(4));
    array.push(&JsValue::from(5));

    let new_array = Array::from(&array.keys().into());

    let mut result = Vec::new();
    new_array.for_each(&mut |i, _, _| result.push(i.as_f64().unwrap()));
    assert_eq!(result, [0.0, 1.0, 2.0, 3.0, 4.0]);
}

#[wasm_bindgen_test]
fn entries() {
    let array = Array::new();
    array.push(&JsValue::from(1));
    array.push(&JsValue::from(2));
    array.push(&JsValue::from(3));
    array.push(&JsValue::from(4));
    array.push(&JsValue::from(5));

    let new_array = Array::from(&array.entries().into());

    new_array.for_each(&mut |a, i, _| {
        assert!(a.is_object());
        let array: Array = a.into();
        assert_eq!(array.shift().as_f64().unwrap(), i as f64);
        assert_eq!(array.shift().as_f64().unwrap(), (i + 1) as f64);
        assert_eq!(array.length(), 0);
    });
}
