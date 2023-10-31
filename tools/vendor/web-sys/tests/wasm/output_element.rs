use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlOutputElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_output() -> HtmlOutputElement;
}

#[wasm_bindgen_test]
fn test_output_element() {
    let output = new_output();
    assert!(
        output.html_for().length() == 0,
        "Our basic <output> should have no html associated with it."
    );
    assert!(
        output.form().is_none(),
        "Our basic <output> should have no form associated with it."
    );

    output.set_name("Calculation result");
    assert_eq!(
        output.name(),
        "Calculation result",
        "Output name should be 'Calculation result'."
    );

    assert_eq!(
        output.type_(),
        "output",
        "Our basic <output> should have an type of 'output'."
    );

    output.set_default_value("27");
    assert_eq!(
        output.default_value(),
        "27",
        "Default output value should be '27'."
    );

    output.set_value("49");
    assert_eq!(output.value(), "49", "Output value should be '49'.");

    // TODO: Fails in Chrome, but not in Firefox.
    //assert!(output.will_validate(), "Output should validate by default (maybe browser dependent?)");

    assert!(
        output.validity().valid(),
        "Our <output>s validity should be true."
    );

    assert!(
        output.validation_message().is_ok(),
        "We should be able to retrieve some validation message from our <output>."
    );

    assert!(output.check_validity(), "Our <output> should be valid.");

    assert!(
        output.report_validity(),
        "Our <output> should report valid."
    );

    output.set_custom_validity("Some scary error message.");

    assert!(
        output.labels().length() == 0,
        "Our basic <output> shouldn't have any labels associated with it."
    );
}
