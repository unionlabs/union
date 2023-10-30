use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlModElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_del() -> HtmlModElement;
    fn new_ins() -> HtmlModElement;
}

#[wasm_bindgen_test]
fn test_mod_elements() {
    let del = new_del();

    del.set_cite("https://www.rust-lang.org/en-US/");
    assert_eq!(
        del.cite(),
        "https://www.rust-lang.org/en-US/",
        "Option should have the cite URI we gave it."
    );

    del.set_date_time("Thu Aug 02 2018 18:02:56 GMT-0500 (Central Daylight Time)");
    assert_eq!(
        del.date_time(),
        "Thu Aug 02 2018 18:02:56 GMT-0500 (Central Daylight Time)",
        "Option should have the date_time we gave it."
    );

    let ins = new_ins();

    ins.set_cite("https://www.rust-lang.org/en-US/");
    assert_eq!(
        ins.cite(),
        "https://www.rust-lang.org/en-US/",
        "Option should have the cite URI we gave it."
    );

    ins.set_date_time("Thu Aug 02 2018 18:02:56 GMT-0500 (Central Daylight Time)");
    assert_eq!(
        ins.date_time(),
        "Thu Aug 02 2018 18:02:56 GMT-0500 (Central Daylight Time)",
        "Option should have the date_time we gave it."
    );
}
