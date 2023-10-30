use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::Headers;

#[wasm_bindgen(module = "/tests/wasm/headers.js")]
extern "C" {
    fn new_headers() -> Headers;
}

#[wasm_bindgen_test]
fn headers() {
    let headers = new_headers();
    assert_eq!(headers.get("foo").unwrap(), None);
    assert_eq!(
        headers.get("content-type").unwrap(),
        Some("text/plain".to_string()),
    );
    assert_eq!(
        headers.get("Content-Type").unwrap(),
        Some("text/plain".to_string()),
    );
    assert!(headers.get("").is_err());
    assert!(headers.set("", "").is_err());
    assert!(headers.set("x", "").is_ok());
    assert_eq!(headers.get("x").unwrap(), Some(String::new()));
    assert!(headers.delete("x").is_ok());
    assert_eq!(headers.get("x").unwrap(), None);
    assert!(headers.append("a", "y").is_ok());
    assert!(headers.append("a", "z").is_ok());
    assert_eq!(headers.get("a").unwrap(), Some("y, z".to_string()));
}
