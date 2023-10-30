use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlTitleElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_title() -> HtmlTitleElement;
}

#[wasm_bindgen_test]
fn title_element() {
    let element = new_title();
    assert_eq!(element.text().unwrap(), "", "Shouldn't have an text");
    assert_eq!(element.set_text("page text").unwrap(), ());
    assert_eq!(element.text().unwrap(), "page text", "Should have an text");
}
