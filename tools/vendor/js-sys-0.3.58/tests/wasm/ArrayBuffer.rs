use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn new() {
    let x = ArrayBuffer::new(42);
    let y: JsValue = x.into();
    assert!(y.is_object());
}

#[wasm_bindgen_test]
fn byte_length() {
    let buf = ArrayBuffer::new(42);
    assert_eq!(buf.byte_length(), 42);
}

#[wasm_bindgen_test]
fn is_view() {
    let x = Uint8Array::new(&JsValue::from(42));
    assert!(ArrayBuffer::is_view(&JsValue::from(x)));
}

#[wasm_bindgen_test]
fn slice() {
    let buf = ArrayBuffer::new(4);
    let slice = buf.slice(2);
    assert!(JsValue::from(slice).is_object());
}

#[wasm_bindgen_test]
fn slice_with_end() {
    let buf = ArrayBuffer::new(4);
    let slice = buf.slice_with_end(1, 2);
    assert!(JsValue::from(slice).is_object());
}

#[wasm_bindgen_test]
fn arraybuffer_inheritance() {
    let buf = ArrayBuffer::new(4);
    assert!(buf.is_instance_of::<ArrayBuffer>());
    assert!(buf.is_instance_of::<Object>());
    let _: &Object = buf.as_ref();
}
