use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlInputElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_input() -> HtmlInputElement;
}

#[wasm_bindgen_test]
fn test_input_element() {
    let element = new_input();
    let location = web_sys::window().unwrap().location().href().unwrap();
    assert_eq!(element.accept(), "", "Shouldn't have an accept");
    element.set_accept("audio/*");
    assert_eq!(element.accept(), "audio/*", "Should have an accept");

    assert_eq!(element.alt(), "", "Shouldn't have an alt");
    element.set_alt("alt text");
    assert_eq!(element.alt(), "alt text", "Should have an alt");

    element.set_type("text");
    assert_eq!(element.autocomplete(), "", "Shouldn't have an autocomplete");
    element.set_autocomplete("on");
    assert_eq!(
        element.autocomplete(),
        "on",
        "Shouldn't have an autocomplete"
    );

    assert!(!element.autofocus(), "Shouldn't have an autofocus");
    element.set_autofocus(true);
    assert!(element.autofocus(), "Should have an autofocus");

    element.set_type("checkbox");
    assert!(
        !element.default_checked(),
        "Shouldn't have an default_checked"
    );
    element.set_default_checked(true);
    assert!(element.default_checked(), "Should have an default_checked");

    /*TODO fix
        assert!(!element.checked(), "Shouldn't be checked");
        element.set_checked(true);
        assert!(element.checked(), "Should be checked");
    */

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

    assert_eq!(element.height(), 0, "Should have no height");
    element.set_height(12);
    assert_eq!(element.height(), 0, "Should have no height"); // Doesn't change, TODO check with get_attribute("height")=="12"

    /*TODO fails in chrome
    element.set_type("checkbox");
    assert!(element.indeterminate(), "Should be indeterminate");
    element.set_checked(true);
    assert!(!element.indeterminate(), "Shouldn't be indeterminate");
    */
    /*TODO add tests
    pub fn indeterminate(&self) -> bool
    pub fn set_indeterminate(&self, indeterminate: bool)
    pub fn input_mode(&self) -> String
    pub fn set_input_mode(&self, input_mode: &str)
    pub fn list(&self) -> Option<HtmlElement>
    pub fn max(&self) -> String
    pub fn set_max(&self, max: &str)
    pub fn max_length(&self) -> i32
    pub fn set_max_length(&self, max_length: i32)
    pub fn min(&self) -> String
    pub fn set_min(&self, min: &str)
    pub fn min_length(&self) -> i32
    pub fn set_min_length(&self, min_length: i32)
    pub fn multiple(&self) -> bool
    pub fn set_multiple(&self, multiple: bool)
    */
    assert_eq!(element.name(), "", "Should not have a name");
    element.set_name("namey");
    assert_eq!(element.name(), "namey", "Should have a name");
    /*TODO add tests
    pub fn pattern(&self) -> String
    pub fn set_pattern(&self, pattern: &str)
    */
    assert_eq!(element.placeholder(), "", "Should not have a placeholder");
    element.set_placeholder("some text");
    assert_eq!(
        element.placeholder(),
        "some text",
        "Should have a placeholder"
    );

    assert!(!element.read_only(), "Should have not be readonly");
    element.set_read_only(true);
    assert!(element.read_only(), "Should be readonly");

    assert!(!element.required(), "Should have not be required");
    element.set_required(true);
    assert!(element.required(), "Should be required");
    /*TODO add tests
    pub fn size(&self) -> u32
    pub fn set_size(&self, size: u32)
    */
    /*TODO fails in chrome
        element.set_type("image");
        assert_eq!(element.src(), "", "Should have no src");
        element.set_value("hey.png");
        assert_eq!(element.src(), "hey.png", "Should have a src");
    */
    /*TODO add tests
    pub fn src(&self) -> String
    pub fn set_src(&self, src: &str)
    pub fn step(&self) -> String
    pub fn set_step(&self, step: &str)
    pub fn type_(&self) -> String
    pub fn set_type(&self, type_: &str)
    pub fn default_value(&self) -> String
    pub fn set_default_value(&self, default_value: &str)
    */
    /*TODO fails in chrome
        assert_eq!(element.value(), "", "Should have no value");
        element.set_value("hey!");
        assert_eq!(element.value(), "hey!", "Should have a value");
    */
    element.set_type("number");
    element.set_value("1");
    assert_eq!(element.value_as_number(), 1.0, "Should have value 1");
    element.set_value_as_number(2.0);
    assert_eq!(element.value(), "2", "Should have value 2");

    assert_eq!(element.width(), 0, "Should have no width");
    element.set_width(12);
    assert_eq!(element.width(), 0, "Should have no width"); // Doesn't change, TODO check with get_attribute("width")=="12"

    assert_eq!(element.will_validate(), false, "Shouldn't validate");
    assert_eq!(
        element.validation_message().unwrap(),
        "",
        "Shouldn't have a value"
    );
    assert_eq!(element.check_validity(), true, "Should be valid");
    assert_eq!(element.report_validity(), true, "Should be valid");
    element.set_custom_validity("Boop"); // Method exists but doesn't impact validity ?!??! TODO look into
    assert_eq!(element.check_validity(), true, "Should be valid");
    assert_eq!(element.report_validity(), true, "Should be valid");
    /*TODO add tests
    pub fn labels(&self) -> Option<NodeList>
    pub fn select(&self)
    pub fn selection_direction(&self) -> Result<Option<String>, JsValue>
    pub fn set_selection_direction(
        &self,
        selection_direction: Option<&str>
    ) -> Result<(), JsValue>
    pub fn set_range_text(&self, replacement: &str) -> Result<(), JsValue>
    pub fn set_selection_range(
        &self,
        start: u32,
        end: u32,
        direction: &str
    ) -> Result<(), JsValue>
    */

    assert_eq!(element.align(), "", "Should have no align");
    element.set_align("left");
    assert_eq!(element.align(), "left", "Should have an align");
    /*TODO add tests
    pub fn use_map(&self) -> String
    pub fn set_use_map(&self, use_map: &str)
    pub fn text_length(&self) -> i32
    pub fn webkitdirectory(&self) -> bool
    pub fn set_webkitdirectory(&self, webkitdirectory: bool)
    pub fn set_focus_state(&self, a_is_focused: bool)
    */
}
