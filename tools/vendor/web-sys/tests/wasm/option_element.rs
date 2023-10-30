use wasm_bindgen_test::*;
use web_sys::HtmlOptionElement;

#[wasm_bindgen_test]
fn test_option_element() {
    let option = HtmlOptionElement::new_with_text_and_value_and_default_selected_and_selected(
        "option_text",
        "option_value",
        false,
        true,
    )
    .unwrap();

    option.set_disabled(true);
    assert_eq!(
        option.disabled(),
        true,
        "Option should be disabled after we set it to be disabled."
    );

    option.set_disabled(false);
    assert_eq!(
        option.disabled(),
        false,
        "Option should not be disabled after we set it to be not-disabled."
    );

    assert!(
        option.form().is_none(),
        "Our option should not be associated with a form."
    );

    option.set_label("Well this truly is a neat option");
    assert_eq!(
        option.label(),
        "Well this truly is a neat option",
        "Option should have the label we gave it."
    );

    option.set_default_selected(true);
    assert_eq!(
        option.default_selected(),
        true,
        "Option should be default_selected after we set it to be default_selected."
    );

    option.set_default_selected(false);
    assert_eq!(
        option.default_selected(),
        false,
        "Option should not be default_selected after we set it to be not default_selected."
    );

    option.set_selected(true);
    assert_eq!(
        option.selected(),
        true,
        "Option should be selected after we set it to be selected."
    );

    option.set_selected(false);
    assert_eq!(
        option.selected(),
        false,
        "Option should not be selected after we set it to be not selected."
    );

    option.set_value("tomato");
    assert_eq!(
        option.value(),
        "tomato",
        "Option should have the value we gave it."
    );

    option.set_text("potato");
    assert_eq!(
        option.text(),
        "potato",
        "Option should have the text we gave it."
    );

    assert_eq!(
        option.index(),
        0,
        "This should be the first option, since there are no other known options."
    );
}
