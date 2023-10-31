use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn new() {
    let error = Error::new("some message");
    assert_eq!(JsValue::from(error.message()), "some message");
}

#[wasm_bindgen_test]
fn new_with_cause() {
    let options = Object::new();
    Reflect::set(
        options.as_ref(),
        &JsValue::from("cause"),
        &JsValue::from("some cause"),
    )
    .unwrap();
    let error = Error::new_with_options("some message", &options);
    assert_eq!(error.cause(), "some cause");
}

#[wasm_bindgen_test]
fn empty_cause() {
    let error = Error::new("test");
    assert_eq!(error.cause(), JsValue::UNDEFINED);
}

#[wasm_bindgen_test]
fn set_cause() {
    let error = Error::new("test");
    error.set_cause(&JsValue::from("different"));
    assert_eq!(error.cause(), "different");
}

#[wasm_bindgen_test]
fn set_message() {
    let error = Error::new("test");
    error.set_message("another");
    assert_eq!(JsValue::from(error.message()), "another");
}

#[wasm_bindgen_test]
fn name() {
    let error = Error::new("test");
    assert_eq!(JsValue::from(error.name()), "Error");
}

#[wasm_bindgen_test]
fn set_name() {
    let error = Error::new("test");
    error.set_name("different");
    assert_eq!(JsValue::from(error.name()), "different");
}

#[wasm_bindgen_test]
fn to_string() {
    let error = Error::new("error message 1");
    assert_eq!(JsValue::from(error.to_string()), "Error: error message 1");
    error.set_name("error_name_1");
    assert_eq!(
        JsValue::from(error.to_string()),
        "error_name_1: error message 1"
    );
}

#[wasm_bindgen_test]
fn error_inheritance() {
    let error = Error::new("test");
    assert!(error.is_instance_of::<Error>());
    assert!(error.is_instance_of::<Object>());
    let _: &Object = error.as_ref();
}
