use js_sys::{Uint8Array, WebAssembly};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/api.js")]
extern "C" {
    fn js_works();
    fn js_eq_works();
    fn assert_null(v: JsValue);
    fn debug_values() -> JsValue;
    fn assert_function_table(a: JsValue, b: usize);
}

#[wasm_bindgen_test]
fn works() {
    js_works();
}

#[wasm_bindgen]
pub fn api_foo() -> JsValue {
    JsValue::from("foo")
}

#[wasm_bindgen]
pub fn api_bar(s: &str) -> JsValue {
    JsValue::from(s)
}

#[wasm_bindgen]
pub fn api_baz() -> JsValue {
    JsValue::from(1.0)
}

#[wasm_bindgen]
pub fn api_baz2(a: &JsValue, b: &JsValue) {
    assert_eq!(a.as_f64(), Some(2.0));
    assert_eq!(b.as_f64(), None);
}

#[wasm_bindgen]
pub fn api_js_null() -> JsValue {
    JsValue::null()
}

#[wasm_bindgen]
pub fn api_js_undefined() -> JsValue {
    JsValue::undefined()
}

#[wasm_bindgen]
pub fn api_test_is_null_undefined(a: &JsValue, b: &JsValue, c: &JsValue) {
    assert!(a.is_null());
    assert!(!a.is_undefined());

    assert!(!b.is_null());
    assert!(b.is_undefined());

    assert!(!c.is_null());
    assert!(!c.is_undefined());
}

#[wasm_bindgen]
pub fn api_get_true() -> JsValue {
    JsValue::from(true)
}

#[wasm_bindgen]
pub fn api_get_false() -> JsValue {
    JsValue::from(false)
}

#[wasm_bindgen]
pub fn api_test_bool(a: &JsValue, b: &JsValue, c: &JsValue) {
    assert_eq!(a.as_bool(), Some(true));
    assert_eq!(format!("{:?}", a), "JsValue(true)");
    assert_eq!(b.as_bool(), Some(false));
    assert_eq!(c.as_bool(), None);
}

#[wasm_bindgen]
pub fn api_mk_symbol() -> JsValue {
    let a = JsValue::symbol(None);
    assert!(a.is_symbol());
    assert_eq!(format!("{:?}", a), "JsValue(Symbol)");
    a
}

#[wasm_bindgen]
pub fn api_mk_symbol2(s: &str) -> JsValue {
    let a = JsValue::symbol(Some(s));
    assert!(a.is_symbol());
    a
}

#[wasm_bindgen]
pub fn api_assert_symbols(a: &JsValue, b: &JsValue) {
    assert!(a.is_symbol());
    assert!(!b.is_symbol());
}

#[wasm_bindgen]
pub fn api_acquire_string(a: &JsValue, b: &JsValue) {
    assert_eq!(a.as_string().unwrap(), "foo");
    assert_eq!(format!("{:?}", a), "JsValue(\"foo\")");
    assert_eq!(b.as_string(), None);
}

#[wasm_bindgen]
pub fn api_acquire_string2(a: &JsValue) -> String {
    a.as_string().unwrap_or("wrong".to_string())
}

#[wasm_bindgen_test]
fn eq_works() {
    js_eq_works();
}

#[wasm_bindgen]
pub fn eq_test(a: &JsValue, b: &JsValue) -> bool {
    a == b
}

#[wasm_bindgen]
#[allow(clippy::eq_op)]
pub fn eq_test1(a: &JsValue) -> bool {
    a == a
}

#[wasm_bindgen(variadic)]
pub fn api_completely_variadic(args: &JsValue) -> JsValue {
    args.into()
}

#[wasm_bindgen(variadic)]
pub fn api_variadic_with_prefixed_params(
    _first: &JsValue,
    _second: &JsValue,
    args: &JsValue,
) -> JsValue {
    args.into()
}

#[wasm_bindgen_test]
fn null_keeps_working() {
    assert_null(JsValue::null());
    assert_null(JsValue::null());
}

#[wasm_bindgen_test]
fn memory_accessor_appears_to_work() {
    let data = 3u32;
    let ptr = &data as *const u32 as u32;

    let my_mem = wasm_bindgen::memory();
    let mem = my_mem.dyn_into::<WebAssembly::Memory>().unwrap();
    let buf = mem.buffer();
    let slice = Uint8Array::new(&buf);
    let mut v = Vec::new();
    slice
        .subarray(ptr, ptr + 4)
        .for_each(&mut |val, _, _| v.push(val));
    assert_eq!(v, [3, 0, 0, 0]);
}

#[wasm_bindgen_test]
fn debug_output() {
    let test_iter = debug_values()
        .dyn_into::<js_sys::Array>()
        .unwrap()
        .values()
        .into_iter();
    let expecteds = vec![
        "JsValue(null)",
        "JsValue(undefined)",
        "JsValue(0)",
        "JsValue(1)",
        "JsValue(true)",
        "JsValue([1, 2, 3])",
        "JsValue(\"string\")",
        "JsValue(Object({\"test\":\"object\"}))",
        "JsValue([1, [2, 3]])",
        "JsValue(Function)",
        "JsValue(Set)",
    ];
    for (test, expected) in test_iter.zip(expecteds) {
        assert_eq!(format!("{:?}", test.unwrap()), expected);
    }
}

#[wasm_bindgen_test]
fn function_table_is() {
    assert_function_table(
        wasm_bindgen::function_table(),
        function_table_lookup as usize,
    );
}

#[no_mangle]
pub extern "C" fn function_table_lookup() {}
