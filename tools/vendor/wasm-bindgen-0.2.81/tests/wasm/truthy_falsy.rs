use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn test_is_truthy() {
    assert_eq!(JsValue::from(0).is_truthy(), false);
    assert_eq!(JsValue::from("".to_string()).is_truthy(), false);
    assert_eq!(JsValue::from(false).is_truthy(), false);
    assert_eq!(JsValue::NULL.is_truthy(), false);
    assert_eq!(JsValue::UNDEFINED.is_truthy(), false);

    assert_eq!(JsValue::from(10).is_truthy(), true);
    assert_eq!(JsValue::from("null".to_string()).is_truthy(), true);
    assert_eq!(JsValue::from(true).is_truthy(), true);
}

#[wasm_bindgen_test]
fn test_is_falsy() {
    assert_eq!(JsValue::from(0).is_falsy(), true);
    assert_eq!(JsValue::from("".to_string()).is_falsy(), true);
    assert_eq!(JsValue::from(false).is_falsy(), true);
    assert_eq!(JsValue::NULL.is_falsy(), true);
    assert_eq!(JsValue::UNDEFINED.is_falsy(), true);

    assert_eq!(JsValue::from(10).is_falsy(), false);
    assert_eq!(JsValue::from("null".to_string()).is_falsy(), false);
    assert_eq!(JsValue::from(true).is_falsy(), false);
}
