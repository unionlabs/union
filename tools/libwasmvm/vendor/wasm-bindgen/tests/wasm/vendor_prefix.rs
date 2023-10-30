use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/vendor_prefix.js")]
extern "C" {
    fn import_me(x: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(vendor_prefix = webkit)]
    type MySpecialApi;
    #[wasm_bindgen(constructor)]
    fn new() -> MySpecialApi;
    #[wasm_bindgen(method)]
    fn foo(this: &MySpecialApi) -> u32;

    #[wasm_bindgen(vendor_prefix = webkit)]
    type MySpecialApi2;
    #[wasm_bindgen(constructor)]
    fn new() -> MySpecialApi2;
    #[wasm_bindgen(method)]
    fn foo(this: &MySpecialApi2) -> u32;

    #[wasm_bindgen(vendor_prefix = a, vendor_prefix = b)]
    type MySpecialApi3;
    #[wasm_bindgen(constructor)]
    fn new() -> MySpecialApi3;
    #[wasm_bindgen(method)]
    fn foo(this: &MySpecialApi3) -> u32;

    // This API does not exist at all;
    // test that Rust gets a chance to catch the error (#2437)
    #[wasm_bindgen(vendor_prefix = a, vendor_prefix = b)]
    type MyMissingApi;
    #[wasm_bindgen(constructor, catch)]
    fn new() -> Result<MyMissingApi, JsValue>;
}

#[wasm_bindgen_test]
pub fn polyfill_works() {
    import_me("foo");

    assert_eq!(MySpecialApi::new().foo(), 123);
    assert_eq!(MySpecialApi2::new().foo(), 124);
    assert_eq!(MySpecialApi3::new().foo(), 125);
    assert!(MyMissingApi::new().is_err());
}
