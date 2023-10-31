use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlQuoteElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_quote() -> HtmlQuoteElement;
}

#[wasm_bindgen_test]
fn test_quote_element() {
    let quote = new_quote();
    quote.set_cite("https://en.wikipedia.org/wiki/Rust_(programming_language)");
    assert_eq!(
        quote.cite(),
        "https://en.wikipedia.org/wiki/Rust_(programming_language)"
    );
}
