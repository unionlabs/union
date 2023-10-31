use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlOListElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_olist() -> HtmlOListElement;
}

#[wasm_bindgen_test]
fn test_olist_element() {
    let olist = new_olist();

    olist.set_reversed(true);
    assert_eq!(
        olist.reversed(),
        true,
        "Olist should be reversed after we set it to be reversed."
    );

    olist.set_reversed(false);
    assert_eq!(
        olist.reversed(),
        false,
        "Olist should not be reversed after we set it to be not reversed."
    );

    olist.set_start(23);
    assert_eq!(
        olist.start(),
        23,
        "Olist should have the start value we gave it."
    );

    olist.set_type("A");
    assert_eq!(
        olist.type_(),
        "A",
        "Olist should be type 'A' after we set it to be type 'A'."
    );

    olist.set_type("I");
    assert_eq!(
        olist.type_(),
        "I",
        "Olist should be type 'I' after we set it to be type 'I'."
    );

    olist.set_compact(true);
    assert_eq!(
        olist.compact(),
        true,
        "Olist should be compact after we set it to be compact."
    );

    olist.set_compact(false);
    assert_eq!(
        olist.compact(),
        false,
        "Olist should not be compact after we set it to be not compact."
    );
}
