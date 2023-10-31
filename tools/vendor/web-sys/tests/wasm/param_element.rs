use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlParamElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_param() -> HtmlParamElement;
}

#[wasm_bindgen_test]
fn test_param_element() {
    let param = new_param();
    param.set_name("color");
    assert_eq!(param.name(), "color", "Name of param should be 'color'.");

    param.set_value("purple");
    assert_eq!(
        param.value(),
        "purple",
        "Value of param should be 'purple'."
    );

    param.set_value_type("ref");
    assert_eq!(
        param.value_type(),
        "ref",
        "Value type of param should be 'ref'."
    );

    param.set_type("text/plain");
    assert_eq!(
        param.type_(),
        "text/plain",
        "Value of param should be 'text/plain'."
    );
}
