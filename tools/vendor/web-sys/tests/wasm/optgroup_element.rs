use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlOptGroupElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_optgroup() -> HtmlOptGroupElement;
}

#[wasm_bindgen_test]
fn test_optgroup_element() {
    let optgroup = new_optgroup();

    optgroup.set_disabled(true);
    assert_eq!(
        optgroup.disabled(),
        true,
        "Optgroup should be disabled after we set it to be disabled."
    );

    optgroup.set_disabled(false);
    assert_eq!(
        optgroup.disabled(),
        false,
        "Optgroup should not be disabled after we set it to be not-disabled."
    );

    optgroup.set_label("Group of options below");
    assert_eq!(
        optgroup.label(),
        "Group of options below",
        "Optgroup should have the label we gave it."
    );
}
