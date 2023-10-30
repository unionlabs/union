use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlMenuItemElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_menuitem() -> HtmlMenuItemElement;
}

#[wasm_bindgen_test]
fn test_menuitem_element() {
    let menuitem = new_menuitem();

    menuitem.set_type("radio");
    assert_eq!(menuitem.type_(), "radio", "Menu item should have the type value we gave it.");

    menuitem.set_label("Menu item label here");
    assert_eq!(menuitem.label(), "Menu item label here", "Menu item should have the label value we gave it.");

    menuitem.set_icon("https://en.wikipedia.org/wiki/Rust_(programming_language)#/media/File:Rust_programming_language_black_logo.svg");
    assert_eq!(menuitem.icon(), "https://en.wikipedia.org/wiki/Rust_(programming_language)#/media/File:Rust_programming_language_black_logo.svg", "Menu item should have the icon value we gave it.");

    menuitem.set_disabled(true);
    assert_eq!(menuitem.disabled(), true, "Menu item should be disabled after we set it to be disabled.");

    menuitem.set_disabled(false);
    assert_eq!(menuitem.disabled(), false, "Menu item should not be disabled after we set it to be not-disabled.");

    menuitem.set_checked(true);
    assert_eq!(menuitem.checked(), true, "Menu item should be checked after we set it to be checked.");

    menuitem.set_checked(false);
    assert_eq!(menuitem.checked(), false, "Menu item should not be checked after we set it to be not-checked.");

    menuitem.set_radiogroup("Radio group name");
    assert_eq!(menuitem.radiogroup(), "Radio group name", "Menu item should have the radiogroup value we gave it.");

    menuitem.set_default_checked(true);
    assert_eq!(menuitem.default_checked(), true, "Menu item should be default_checked after we set it to be default_checked.");

    menuitem.set_default_checked(false);
    assert_eq!(menuitem.default_checked(), false, "Menu item should not be default_checked after we set it to be not default_checked.");
}
