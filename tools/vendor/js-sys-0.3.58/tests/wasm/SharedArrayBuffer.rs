use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/SharedArrayBuffer.js")]
extern "C" {
    fn is_shared_array_buffer_supported() -> bool;
}

#[wasm_bindgen_test]
fn new() {
    if !is_shared_array_buffer_supported() {
        return;
    }
    let x = SharedArrayBuffer::new(42);
    let y: JsValue = x.into();
    assert!(y.is_object());
}

#[wasm_bindgen_test]
fn byte_length() {
    if !is_shared_array_buffer_supported() {
        return;
    }
    let buf = SharedArrayBuffer::new(42);
    assert_eq!(buf.byte_length(), 42);
}

#[wasm_bindgen_test]
fn slice() {
    if !is_shared_array_buffer_supported() {
        return;
    }
    let buf = SharedArrayBuffer::new(4);
    let slice = buf.slice(2);
    assert!(JsValue::from(slice).is_object());
}

#[wasm_bindgen_test]
fn slice_with_end() {
    if !is_shared_array_buffer_supported() {
        return;
    }
    let buf = SharedArrayBuffer::new(4);
    let slice = buf.slice_with_end(1, 2);
    assert!(JsValue::from(slice).is_object());
}

#[wasm_bindgen_test]
fn sharedarraybuffer_inheritance() {
    if !is_shared_array_buffer_supported() {
        return;
    }
    let buf = SharedArrayBuffer::new(4);
    assert!(buf.is_instance_of::<SharedArrayBuffer>());
    assert!(buf.is_instance_of::<Object>());
    let _: &Object = buf.as_ref();
}
