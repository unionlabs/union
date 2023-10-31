use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/Proxy.js")]
extern "C" {
    fn proxy_target() -> JsValue;
    fn proxy_handler() -> Object;

    type Custom;
    #[wasm_bindgen(method, getter, structural, catch)]
    fn a(this: &Custom) -> Result<u32, JsValue>;
    #[wasm_bindgen(method, getter, structural, catch)]
    fn b(this: &Custom) -> Result<u32, JsValue>;

    type RevocableResult;
    #[wasm_bindgen(method, getter, structural)]
    fn proxy(this: &RevocableResult) -> JsValue;
    #[wasm_bindgen(method, getter, structural)]
    fn revoke(this: &RevocableResult) -> Function;
}

#[wasm_bindgen_test]
fn new() {
    let proxy = Proxy::new(&proxy_target(), &proxy_handler());
    let proxy = Custom::from(JsValue::from(proxy));
    assert_eq!(proxy.a().unwrap(), 100);
    assert_eq!(proxy.b().unwrap(), 37);
}

#[wasm_bindgen_test]
fn revocable() {
    let result = Proxy::revocable(&proxy_target(), &proxy_handler());
    let result = RevocableResult::from(JsValue::from(result));
    let proxy = result.proxy();
    let revoke = result.revoke();

    let obj = Custom::from(proxy);
    assert_eq!(obj.a().unwrap(), 100);
    assert_eq!(obj.b().unwrap(), 37);
    revoke.apply(&JsValue::undefined(), &Array::new()).unwrap();
    assert!(obj.a().is_err());
    assert!(obj.b().is_err());
    assert!(JsValue::from(obj).is_object());
}
