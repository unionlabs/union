use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::XPathResult;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_xpath_result() -> XPathResult;
}

#[wasm_bindgen_test]
fn test_xpath_result() {
    let xpath_result = new_xpath_result();
    assert_eq!(
        xpath_result.result_type(),
        XPathResult::UNORDERED_NODE_ITERATOR_TYPE
    );
    assert_eq!(xpath_result.invalid_iterator_state(), false);
    assert_eq!(
        xpath_result
            .iterate_next()
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap(),
        "tomato"
    );
}
