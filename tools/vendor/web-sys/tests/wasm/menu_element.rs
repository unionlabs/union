use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlMenuElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_menu() -> HtmlMenuElement;
}

#[wasm_bindgen_test]
fn test_menu_element() {
    let menu = new_menu();

    menu.set_type("toolbar");
    assert_eq!(menu.type_(), "toolbar", "Menu should have the type value we gave it.");

    menu.set_label("Menu label here");
    assert_eq!(menu.label(), "Menu label here", "Menu should have the label value we gave it.");

    menu.set_compact(true);
    assert_eq!(menu.compact(), true, "Menu should be compact after we set it to be compact.");

    menu.set_compact(false);
    assert_eq!(menu.compact(), false, "Menu should not be compact after we set it to be not-compact.");
}
