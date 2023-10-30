use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen_test::*;
use web_sys::HtmlAnchorElement;
#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_a() -> HtmlAnchorElement;
}

#[wasm_bindgen_test]
fn test_anchor_element() {
    // This test is to make sure there is no weird mutability going on.
    let buf = vec![1, 2, 3, 255];
    let image_data = web_sys::ImageData::new_with_u8_clamped_array(Clamped(&buf), 1).unwrap();
    let mut data = image_data.data();
    data[1] = 4;
    assert_eq!(buf[1], 2);
}
