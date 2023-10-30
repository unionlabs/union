use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlMetaElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_meta() -> HtmlMetaElement;
}

#[wasm_bindgen_test]
fn test_meter_element() {
    let meta = new_meta();

    meta.set_name("keywords");
    assert_eq!(
        meta.name(),
        "keywords",
        "Meta should have the name value we gave it."
    );

    meta.set_http_equiv("content-type");
    assert_eq!(
        meta.http_equiv(),
        "content-type",
        "Meta should have the http_equiv value we gave it."
    );

    meta.set_content("HTML, CSS, XML, JavaScript");
    assert_eq!(
        meta.content(),
        "HTML, CSS, XML, JavaScript",
        "Meta should have the content value we gave it."
    );

    meta.set_scheme("text");
    assert_eq!(
        meta.scheme(),
        "text",
        "Meta should have the scheme value we gave it."
    );
}
