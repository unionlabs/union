use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{HtmlButtonElement, HtmlFormElement, Node};

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_button() -> HtmlButtonElement;
    fn new_form() -> HtmlFormElement;
}

#[wasm_bindgen_test]
fn test_button_element() {
    let element = new_button();
    let location = web_sys::window().unwrap().location().href().unwrap();
    assert!(!element.autofocus(), "Shouldn't have autofocus");
    element.set_autofocus(true);
    assert!(element.autofocus(), "Should have autofocus");

    assert!(!element.disabled(), "Shouldn't be disabled");
    element.set_disabled(true);
    assert!(element.disabled(), "Should be disabled");

    match element.form() {
        None => assert!(true, "Shouldn't have a form"),
        _ => assert!(false, "Shouldn't have a form"),
    };

    assert_eq!(
        element.form_action(),
        location,
        "Should have the pages location"
    );
    element.set_form_action("http://boop.com/");
    assert_eq!(
        element.form_action(),
        "http://boop.com/",
        "Should have a form_action"
    );

    assert_eq!(element.form_enctype(), "", "Should have no enctype");
    element.set_form_enctype("text/plain");
    assert_eq!(
        element.form_enctype(),
        "text/plain",
        "Should have a plain text enctype"
    );

    assert_eq!(element.form_method(), "", "Should have no method");
    element.set_form_method("POST");
    assert_eq!(element.form_method(), "post", "Should have a POST method");

    assert!(!element.form_no_validate(), "Should validate");
    element.set_form_no_validate(true);
    assert!(element.form_no_validate(), "Should not validate");

    assert_eq!(element.form_target(), "", "Should have no target");
    element.set_form_target("_blank");
    assert_eq!(
        element.form_target(),
        "_blank",
        "Should have a _blank target"
    );

    assert_eq!(element.name(), "", "Shouldn't have a name");
    element.set_name("button-name");
    assert_eq!(element.name(), "button-name", "Should have a name");

    assert_eq!(element.type_(), "submit", "Shouldn't have a type");
    element.set_type("reset");
    assert_eq!(element.type_(), "reset", "Should have a reset type");

    assert_eq!(element.value(), "", "Shouldn't have a value");
    element.set_value("value1");
    assert_eq!(element.value(), "value1", "Should have a value");

    assert_eq!(element.will_validate(), false, "Shouldn't validate");
    assert_eq!(
        element.validation_message().unwrap(),
        "",
        "Shouldn't have a value"
    );
    assert_eq!(element.check_validity(), true, "Should be valid");
    assert_eq!(element.report_validity(), true, "Should be valid");
    element.set_custom_validity("Boop"); // Method exists but doesn't impact validity
    assert_eq!(element.check_validity(), true, "Should be valid");
    assert_eq!(element.report_validity(), true, "Should be valid");

    assert_eq!(
        element.labels().length(),
        0,
        "Should return a node list with no elements"
    );
}

#[wasm_bindgen_test]
fn test_button_element_in_form() {
    let button = new_button();
    button.set_type("reset");
    let form = new_form();
    form.set_name("test-form");

    // TODO: implement `Clone` for types in `web_sys` to make this easier.
    let button = JsValue::from(button);
    let as_node = Node::from(button.clone());
    Node::from(JsValue::from(form))
        .append_child(&as_node)
        .unwrap();

    let element = HtmlButtonElement::from(button);
    match element.form() {
        None => assert!(false, "Should have a form"),
        Some(form) => {
            assert!(true, "Should have a form");
            assert_eq!(
                form.name(),
                "test-form",
                "Form should have a name of test-form"
            );
        }
    };
    assert_eq!(element.type_(), "reset", "Should have a type");
}
