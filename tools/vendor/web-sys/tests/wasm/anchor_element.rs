use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlAnchorElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_a() -> HtmlAnchorElement;
}

#[wasm_bindgen_test]
fn test_anchor_element() {
    let element = new_a();
    assert_eq!(element.target(), "", "Shouldn't have a target");
    element.set_target("_blank");
    assert_eq!(element.target(), "_blank", "Should have a target");

    assert_eq!(element.download(), "", "Shouldn't have a download");
    element.set_download("boop.png");
    assert_eq!(element.download(), "boop.png", "Should have a download");

    assert_eq!(element.ping(), "", "Shouldn't have a ping");
    element.set_ping("boop");
    assert_eq!(element.ping(), "boop", "Should have a ping");

    assert_eq!(element.rel(), "", "Shouldn't have a rel");
    element.set_rel("boop");
    assert_eq!(element.rel(), "boop", "Should have a rel");

    assert_eq!(
        element.referrer_policy(),
        "",
        "Shouldn't have a referrer_policy"
    );
    element.set_referrer_policy("origin");
    assert_eq!(
        element.referrer_policy(),
        "origin",
        "Should have a referrer_policy"
    );

    assert_eq!(element.hreflang(), "", "Shouldn't have a hreflang");
    element.set_hreflang("en-us");
    assert_eq!(element.hreflang(), "en-us", "Should have a hreflang");

    assert_eq!(element.type_(), "", "Shouldn't have a type");
    element.set_type("text/plain");
    assert_eq!(element.type_(), "text/plain", "Should have a type");

    assert_eq!(element.text().unwrap(), "", "Shouldn't have a text");
    element.set_text("Click me!").unwrap();
    assert_eq!(element.text().unwrap(), "Click me!", "Should have a text");

    assert_eq!(element.coords(), "", "Shouldn't have a coords");
    element.set_coords("1,2,3");
    assert_eq!(element.coords(), "1,2,3", "Should have a coords");

    assert_eq!(element.charset(), "", "Shouldn't have a charset");
    element.set_charset("thing");
    assert_eq!(element.charset(), "thing", "Should have a charset");

    assert_eq!(element.name(), "", "Shouldn't have a name");
    element.set_name("thing");
    assert_eq!(element.name(), "thing", "Should have a name");

    assert_eq!(element.rev(), "", "Shouldn't have a rev");
    element.set_rev("thing");
    assert_eq!(element.rev(), "thing", "Should have a rev");

    assert_eq!(element.shape(), "", "Shouldn't have a shape");
    element.set_shape("thing");
    assert_eq!(element.shape(), "thing", "Should have a shape");
}
