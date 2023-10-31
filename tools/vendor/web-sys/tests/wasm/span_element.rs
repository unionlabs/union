use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlSpanElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_span() -> HtmlSpanElement;
}

#[wasm_bindgen_test]
fn test_span_element() {
    let _element = new_span();
    assert!(true, "Span doesn't have an interface");
}
