use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlSlotElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_slot() -> HtmlSlotElement;
}

#[wasm_bindgen_test]
fn test_slot_element() {
    let _slot = new_slot();
    // TODO: Test fails in Firefox, but not in Chrome.  Error in Firefox is 'ReferenceError: HTMLSlotElement is not defined'.  https://w3c-test.org/shadow-dom/HTMLSlotElement-interface.html
    // slot.set_name("root_separator");
    // assert_eq!(slot.name(), "root_separator", "Slot name should 'root_separator'.");
}
