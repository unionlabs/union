use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlPreElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_pre() -> HtmlPreElement;
}

#[wasm_bindgen_test]
fn test_pre_element() {
    let pre = new_pre();
    pre.set_width(150);
    assert_eq!(pre.width(), 150, "Pre width should be 150.");
}
