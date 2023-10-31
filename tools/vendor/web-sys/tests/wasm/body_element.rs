use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlBodyElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_body() -> HtmlBodyElement;
}

#[wasm_bindgen_test]
fn test_body_element() {
    let element = new_body();
    assert_eq!(element.text(), "", "Shouldn't have a text");
    element.set_text("boop");
    assert_eq!(element.text(), "boop", "Should have a text");

    // Legacy color setting
    assert_eq!(element.link(), "", "Shouldn't have a link");
    element.set_link("blue");
    assert_eq!(element.link(), "blue", "Should have a link");

    assert_eq!(element.v_link(), "", "Shouldn't have a v_link");
    element.set_v_link("purple");
    assert_eq!(element.v_link(), "purple", "Should have a v_link");

    assert_eq!(element.a_link(), "", "Shouldn't have a a_link");
    element.set_a_link("purple");
    assert_eq!(element.a_link(), "purple", "Should have a a_link");

    assert_eq!(element.bg_color(), "", "Shouldn't have a bg_color");
    element.set_bg_color("yellow");
    assert_eq!(element.bg_color(), "yellow", "Should have a bg_color");

    assert_eq!(element.background(), "", "Shouldn't have a background");
    element.set_background("image");
    assert_eq!(element.background(), "image", "Should have a background");
}
