use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlHeadElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_head() -> HtmlHeadElement;
}

#[wasm_bindgen_test]
fn test_head_element() {
    let _element = new_head();
    assert!(true, "Head doesn't have an interface");
}
