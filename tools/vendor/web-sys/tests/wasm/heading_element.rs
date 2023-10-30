use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlHeadingElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_heading() -> HtmlHeadingElement;
}

#[wasm_bindgen_test]
fn heading_element() {
    let element = new_heading();
    assert_eq!(element.align(), "", "Shouldn't have an align");
    element.set_align("justify");
    assert_eq!(element.align(), "justify", "Should have an align");
}
