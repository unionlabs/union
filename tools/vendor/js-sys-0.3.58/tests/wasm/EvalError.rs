use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

// Note: This error is not thrown any more, so there are no tests that will generate this error.
// Instead we just have to manually construct it

#[wasm_bindgen_test]
fn new() {
    let error = EvalError::new("some message");
    let base_error: &Error = error.dyn_ref().unwrap();
    assert_eq!(JsValue::from(base_error.message()), "some message");
}

#[wasm_bindgen_test]
fn set_message() {
    let error = EvalError::new("test");
    let base_error: &Error = error.dyn_ref().unwrap();
    base_error.set_message("another");
    assert_eq!(JsValue::from(base_error.message()), "another");
}

#[wasm_bindgen_test]
fn name() {
    let error = EvalError::new("test");
    let base_error: &Error = error.dyn_ref().unwrap();
    assert_eq!(JsValue::from(base_error.name()), "EvalError");
}

#[wasm_bindgen_test]
fn set_name() {
    let error = EvalError::new("test");
    let base_error: &Error = error.dyn_ref().unwrap();
    base_error.set_name("different");
    assert_eq!(JsValue::from(base_error.name()), "different");
}

#[wasm_bindgen_test]
fn to_string() {
    let error = EvalError::new("error message 1");
    let base_error: &Error = error.dyn_ref().unwrap();
    assert_eq!(
        JsValue::from(base_error.to_string()),
        "EvalError: error message 1"
    );
    base_error.set_name("error_name_1");
    assert_eq!(
        JsValue::from(base_error.to_string()),
        "error_name_1: error message 1"
    );
}

#[wasm_bindgen_test]
fn evalerror_inheritance() {
    let error = EvalError::new("some message");
    assert!(error.is_instance_of::<EvalError>());
    assert!(error.is_instance_of::<Error>());
    assert!(error.is_instance_of::<Object>());
    let _: &Error = error.as_ref();
    let _: &Object = error.as_ref();
}
