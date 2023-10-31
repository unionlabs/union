use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlHtmlElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_html() -> HtmlHtmlElement;
}

#[wasm_bindgen_test]
fn test_html_html_element() {
    let element = new_html();
    assert_eq!(element.version(), "", "Shouldn't have a version");
    element.set_version("4");
    assert_eq!(element.version(), "4", "Should have a version");
}
