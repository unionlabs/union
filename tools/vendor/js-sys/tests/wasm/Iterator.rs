use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/Iterator.js")]
extern "C" {
    fn get_iterable() -> JsValue;

    fn get_not_iterable() -> JsValue;

    fn get_symbol_iterator_throws() -> JsValue;

    fn get_symbol_iterator_not_function() -> JsValue;

    fn get_symbol_iterator_returns_not_object() -> JsValue;

    fn get_symbol_iterator_returns_object_without_next() -> JsValue;
}

#[wasm_bindgen_test]
fn try_iter_handles_iteration_protocol() {
    assert_eq!(
        try_iter(&get_iterable())
            .unwrap()
            .unwrap()
            .map(|x| x.unwrap().as_string().unwrap())
            .collect::<Vec<_>>(),
        vec!["one", "two", "three"]
    );

    assert!(try_iter(&get_not_iterable()).unwrap().is_none());
    assert!(try_iter(&get_symbol_iterator_throws()).is_err());
    assert!(try_iter(&get_symbol_iterator_not_function())
        .unwrap()
        .is_none());
    assert!(try_iter(&get_symbol_iterator_returns_not_object())
        .unwrap()
        .is_none());
    assert!(try_iter(&get_symbol_iterator_returns_object_without_next())
        .unwrap()
        .is_none());
}
