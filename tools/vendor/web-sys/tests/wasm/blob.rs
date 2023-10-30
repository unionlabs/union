use js_sys::{Array, ArrayBuffer};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::Blob;

#[wasm_bindgen(module = "/tests/wasm/blob.js")]
extern "C" {
    fn new_blob() -> Blob;
}

#[wasm_bindgen_test]
fn test_blob_from_js() {
    let blob = new_blob();
    assert!(blob.is_instance_of::<Blob>());
    assert_eq!(blob.size(), 3.0);
}

#[wasm_bindgen_test]
fn test_blob_from_bytes() {
    let bytes = Array::new();
    bytes.push(&1.into());
    bytes.push(&2.into());
    bytes.push(&3.into());

    let blob = Blob::new_with_u8_array_sequence(&bytes.into()).unwrap();
    assert!(blob.is_instance_of::<Blob>());
    assert_eq!(blob.size(), 3.0);
}

#[wasm_bindgen_test]
fn test_blob_empty() {
    let blob = Blob::new().unwrap();
    assert!(blob.is_instance_of::<Blob>());
    assert_eq!(blob.size(), 0.0);
}

#[wasm_bindgen_test]
async fn test_blob_array_buffer() {
    let bytes = Array::new();
    bytes.push(&1.into());
    bytes.push(&2.into());
    bytes.push(&3.into());

    let blob = Blob::new_with_u8_array_sequence(&bytes.into()).unwrap();

    let buffer: ArrayBuffer = JsFuture::from(blob.array_buffer()).await.unwrap().into();

    assert!(blob.is_instance_of::<Blob>());
    assert!(buffer.is_instance_of::<ArrayBuffer>());
    assert_eq!(blob.size(), buffer.byte_length() as f64);
}

#[wasm_bindgen_test]
async fn test_blob_text() {
    let strings = Array::new();
    strings.push(&"hello".into());

    let blob = Blob::new_with_str_sequence(&strings.into()).unwrap();
    let string = JsFuture::from(blob.text()).await.unwrap();

    assert!(blob.is_instance_of::<Blob>());
    assert_eq!(string, "hello")
}
