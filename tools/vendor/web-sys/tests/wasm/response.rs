use js_sys::{ArrayBuffer, DataView, Object, Promise, Reflect, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Headers, Response, ResponseInit};

#[wasm_bindgen(module = "/tests/wasm/response.js")]
extern "C" {
    fn new_response() -> Response;
    fn get_wasm_imports() -> Object;
}

#[wasm_bindgen_test]
fn test_response_from_js() {
    let response = new_response();
    assert!(!response.ok());
    assert!(!response.redirected());
    assert_eq!(response.status(), 501);
}

#[wasm_bindgen_test]
async fn test_response_from_bytes() {
    let mut bytes: [u8; 3] = [1, 3, 5];
    let response = Response::new_with_opt_u8_array(Some(&mut bytes)).unwrap();
    assert!(response.ok());
    assert_eq!(response.status(), 200);

    let buf_promise = response.array_buffer().unwrap();
    let buf_val = JsFuture::from(buf_promise).await.unwrap();
    assert!(buf_val.is_instance_of::<ArrayBuffer>());
    let array_buf: ArrayBuffer = buf_val.dyn_into().unwrap();
    let data_view = DataView::new(&array_buf, 0, bytes.len());
    for (i, byte) in bytes.iter().enumerate() {
        assert_eq!(&data_view.get_uint8(i), byte);
    }
}

#[wasm_bindgen_test]
async fn test_response_from_other_body() {
    let input = "Hello, world!";
    let response_a = Response::new_with_opt_str(Some(input)).unwrap();
    let body = response_a.body();
    let response_b = Response::new_with_opt_readable_stream(body.as_ref()).unwrap();
    let output = JsFuture::from(response_b.text().unwrap()).await.unwrap();
    assert_eq!(JsValue::from_str(input), output);
}

// Because it relies on `Response`, this can't go in `js-sys`, so put it here instead.
#[wasm_bindgen_test]
async fn wasm_instantiate_streaming() {
    // Taken from `crates/js-sys/tests/wasm/WebAssembly.js`.
    let mut wasm = *b"\x00asm\x01\x00\x00\x00\x01\x08\x02`\x01\x7f\x00`\x00\x00\x02\x19\x01\x07imports\rimported_func\x00\x00\x03\x02\x01\x01\x07\x11\x01\rexported_func\x00\x01\n\x08\x01\x06\x00A*\x10\x00\x0b";

    let headers = Headers::new().unwrap();
    headers.append("Content-Type", "application/wasm").unwrap();
    let response = Response::new_with_opt_u8_array_and_init(
        Some(&mut wasm),
        ResponseInit::new().headers(&headers),
    )
    .unwrap();
    let response = Promise::resolve(&response);
    let imports = get_wasm_imports();
    let p = WebAssembly::instantiate_streaming(&response, &imports);
    let obj = JsFuture::from(p).await.unwrap();
    assert!(Reflect::get(obj.as_ref(), &"instance".into())
        .unwrap()
        .is_instance_of::<WebAssembly::Instance>());
}
