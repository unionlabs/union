use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlSelectElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_select_with_food_opts() -> HtmlSelectElement;
}

#[wasm_bindgen_test]
fn test_select_element() {
    // Creates a select with four options.  Options are ["tomato", "potato", "orange", "apple"], where
    // the string is the .value and .text of each option.
    let select = new_select_with_food_opts();
    select.set_autofocus(true);
    assert_eq!(
        select.autofocus(),
        true,
        "Select element should have a true autofocus property."
    );

    select.set_autofocus(false);
    assert_eq!(
        select.autofocus(),
        false,
        "Select element should have a false autofocus property."
    );

    //    TODO: This test currently fails on Firefox, but not Chrome.  In Firefox, even though we select.set_autocomplete(), select.autocomplete() yields an empty String.
    //    select.set_autocomplete("tomato");
    //    assert_eq!(select.autocomplete(), "tomato", "Select element should have a 'tomato' autocomplete property.");

    select.set_disabled(true);
    assert_eq!(
        select.disabled(),
        true,
        "Select element should be disabled."
    );

    select.set_disabled(false);
    assert_eq!(
        select.disabled(),
        false,
        "Select element should not be disabled."
    );

    assert!(
        select.form().is_none(),
        "Select should not be associated with a form."
    );

    select.set_multiple(false);
    assert_eq!(
        select.multiple(),
        false,
        "Select element should have a false multiple property."
    );

    select.set_multiple(true);
    assert_eq!(
        select.multiple(),
        true,
        "Select element should have a true multiple property."
    );

    select.set_name("potato");
    assert_eq!(
        select.name(),
        "potato",
        "Select element should have a name property of 'potato'."
    );

    select.set_required(true);
    assert_eq!(
        select.required(),
        true,
        "Select element should be required."
    );

    select.set_required(false);
    assert_eq!(
        select.required(),
        false,
        "Select element should not be required."
    );

    select.set_size(432);
    assert_eq!(
        select.size(),
        432,
        "Select element should have a size property of 432."
    );

    // Default type seems to be "select-multiple" for the browsers I tested, but there's no guarantee
    // on this, so let's just make sure we get back something here.
    assert!(
        select.type_().len() > 0,
        "Select element should have some type."
    );

    assert!(
        select.options().length() == 4,
        "Select element should have four options."
    );

    select.set_length(12);
    assert_eq!(
        select.length(),
        12,
        "Select element should have a length of 12."
    );
    // Reset the length to four, as that's how many options we actually have.
    select.set_length(4);

    assert!(
        select
            .named_item("this should definitely find nothing")
            .is_none(),
        "Shouldn't be able to find a named item with the given string."
    );

    assert!(
        select.selected_options().length() == 1,
        "One option should be selected by default, just by way of having items."
    );

    select.set_selected_index(2);
    assert_eq!(
        select.selected_index(),
        2,
        "Select element should have a selected index of 2."
    );

    // Quote from docs: The value property sets or returns the value of the selected option in a drop-down list.
    select.set_value("tomato"); // Select the "tomato" option
    assert_eq!(
        select.value(),
        "tomato",
        "Select element should have no selected value."
    );

    // This might be browser dependent, potentially rendering this test useless?  Worked fine in Chrome and Firefox for now.
    assert_eq!(
        select.will_validate(),
        true,
        "Select element should not validate by default."
    );

    assert!(
        select.validation_message().is_ok(),
        "Select element should retrieve a validation message."
    );

    assert!(
        select.validity().valid(),
        "Our basic select should be valid."
    );

    assert!(
        select.check_validity(),
        "Our basic select should check out as valid."
    );

    assert!(
        select.report_validity(),
        "Our basic select should report valid."
    );

    select.set_custom_validity("Some custom validity error.");

    assert!(
        select.labels().length() == 0,
        "There should be no labels associated with our select element."
    );

    // TODO: This test won't work until this bug is fixed: https://www.w3.org/Bugs/Public/show_bug.cgi?id=20720.  Sometime in the future, either remove this test or uncomment after bug is fixed.
    // assert!(select.named_item("tomato").is_some(), "Should be able to find the 'tomato' option before removing it.");
    // select.remove(0);
    // assert!(select.named_item("tomato").is_none(), "Shouldn't be able to find the 'tomato' option after removing it.")
    // TODO: As a result, we are missing a test for the remove() method.
}
