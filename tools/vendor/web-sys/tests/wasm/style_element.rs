use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlStyleElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_style() -> HtmlStyleElement;
}

#[wasm_bindgen_test]
fn test_style_element() {
    let element = new_style();
    assert!(!element.disabled(), "Should be disabled");
    element.set_disabled(true);
    assert!(!element.disabled(), "Should be disabled"); // Not sure why this is but Chrome in Firefox behabe the same

    assert_eq!(element.type_(), "", "Shouldn't have a type");
    element.set_type("text/css");
    assert_eq!(element.type_(), "text/css", "Should have a type");

    assert_eq!(element.media(), "", "Shouldn't have a media");
    element.set_media("screen, print");
    assert_eq!(element.media(), "screen, print", "Should have a media");
}
