use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn uri_error() {
    let error = UriError::new("msg");
    assert!(error.is_instance_of::<UriError>());
    assert!(error.is_instance_of::<Error>());
    assert!(error.is_instance_of::<Object>());
    let _: &Error = error.as_ref();
    let _: &Object = error.as_ref();

    let base: &Error = error.as_ref();
    assert_eq!(JsValue::from(base.message()), "msg");
}
