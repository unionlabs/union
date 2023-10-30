use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlHrElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_hr() -> HtmlHrElement;
}

#[wasm_bindgen_test]
fn test_hr_element() {
    let hr = new_hr();
    hr.set_color("blue");
    assert_eq!(hr.color(), "blue");

    hr.set_width("128");
    assert_eq!(hr.width(), "128");

    hr.set_width("256");
    assert_eq!(hr.width(), "256");

    hr.set_no_shade(true);
    assert_eq!(hr.no_shade(), true);
}
