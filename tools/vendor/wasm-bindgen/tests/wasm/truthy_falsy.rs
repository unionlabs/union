use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn test_is_truthy() {
    assert!(!JsValue::from(0).is_truthy());
    assert!(!JsValue::from("".to_string()).is_truthy());
    assert!(!JsValue::from(false).is_truthy());
    assert!(!JsValue::NULL.is_truthy());
    assert!(!JsValue::UNDEFINED.is_truthy());

    assert!(JsValue::from(10).is_truthy());
    assert!(JsValue::from("null".to_string()).is_truthy());
    assert!(JsValue::from(true).is_truthy());
}

#[wasm_bindgen_test]
fn test_is_falsy() {
    assert!(JsValue::from(0).is_falsy());
    assert!(JsValue::from("".to_string()).is_falsy());
    assert!(JsValue::from(false).is_falsy());
    assert!(JsValue::NULL.is_falsy());
    assert!(JsValue::UNDEFINED.is_falsy());

    assert!(!JsValue::from(10).is_falsy());
    assert!(!JsValue::from("null".to_string()).is_falsy());
    assert!(!JsValue::from(true).is_falsy());
}
