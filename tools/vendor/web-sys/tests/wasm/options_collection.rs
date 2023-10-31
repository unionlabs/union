use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlOptionsCollection;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_food_options_collection() -> HtmlOptionsCollection;
}

#[wasm_bindgen_test]
fn test_options_collection() {
    let opt_collection = new_food_options_collection();

    assert!(
        opt_collection.length() == 4,
        "Our option collection should have four options."
    );
    assert!(
        opt_collection.remove(0).is_ok(),
        "We should be able to successfully remove an element from an option collection."
    );
    assert!(
        opt_collection.length() == 3,
        "Our option collection should have three options after removing one."
    );

    assert!(
        opt_collection.set_selected_index(1).is_ok(),
        "Should be able to set the selected index of an option collection if it is valid."
    );
    assert_eq!(
        opt_collection.selected_index().unwrap(),
        1,
        "The second option should be selected in our option collection."
    );

    opt_collection.set_length(1234);
    assert_eq!(
        opt_collection.length(),
        1234,
        "Our option collections length should update after being set to 1234."
    );
}
