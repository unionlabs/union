use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlDivElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_div() -> HtmlDivElement;
}

#[wasm_bindgen_test]
fn test_div_element() {
    let element = new_div();
    assert_eq!(element.align(), "", "Shouldn't have a align");
    element.set_align("right");
    assert_eq!(element.align(), "right", "Should have a align");
}
