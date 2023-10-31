use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use web_sys::HtmlElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_html() -> HtmlElement;
}

#[wasm_bindgen_test]
fn test_html_element() {
    let element = new_html();
    assert!(element.is_instance_of::<HtmlElement>());

    assert_eq!(element.title(), "", "Shouldn't have a title");
    element.set_title("boop");
    assert_eq!(element.title(), "boop", "Should have a title");

    assert_eq!(element.lang(), "", "Shouldn't have a lang");
    element.set_lang("en-us");
    assert_eq!(element.lang(), "en-us", "Should have a lang");

    assert_eq!(element.dir(), "", "Shouldn't have a dir");
    element.set_dir("ltr");
    assert_eq!(element.dir(), "ltr", "Should have a dir");

    assert_eq!(element.inner_text(), "", "Shouldn't have inner_text");
    element.set_inner_text("hey");
    assert_eq!(element.inner_text(), "hey", "Should have inner_text");

    assert!(!element.hidden(), "Shouldn't be hidden");
    element.set_hidden(true);
    assert!(element.hidden(), "Should be hidden");

    assert_eq!(
        element.class_list().get(0),
        None,
        "Shouldn't have class at index 0"
    );
    element.class_list().add_2("a", "b").unwrap();
    assert_eq!(
        element.class_list().get(0).unwrap(),
        "a",
        "Should have class at index 0"
    );
    assert_eq!(
        element.class_list().get(1).unwrap(),
        "b",
        "Should have class at index 1"
    );
    assert_eq!(
        element.class_list().get(2),
        None,
        "Shouldn't have class at index 2"
    );

    assert_eq!(element.dataset().get("id"), None, "Shouldn't have data-id");
    element.dataset().set("id", "123").unwrap();
    assert_eq!(
        element.dataset().get("id").unwrap(),
        "123",
        "Should have data-id"
    );

    assert_eq!(
        element.style().get(0),
        None,
        "Shouldn't have style property name at index 0"
    );
    element
        .style()
        .set_property("background-color", "red")
        .unwrap();
    assert_eq!(
        element.style().get(0).unwrap(),
        "background-color",
        "Should have style property at index 0"
    );
    assert_eq!(
        element
            .style()
            .get_property_value("background-color")
            .unwrap(),
        "red",
        "Should have style property"
    );

    // TODO add a click handler here
    element.click();

    assert_eq!(element.tab_index(), -1, "Shouldn't be tab_index");
    element.set_tab_index(1);
    assert_eq!(element.tab_index(), 1, "Should be tab_index");

    // TODO add a focus handler here
    assert_eq!(element.focus().unwrap(), (), "No result");

    // TODO add a blur handler here
    assert_eq!(element.blur().unwrap(), (), "No result");

    assert_eq!(element.access_key(), "", "Shouldn't have a access_key");
    element.set_access_key("a");
    assert_eq!(element.access_key(), "a", "Should have a access_key");

    // TODO add test for access_key_label

    assert!(!element.draggable(), "Shouldn't be draggable");
    element.set_draggable(true);
    assert!(element.draggable(), "Should be draggable");

    assert_eq!(
        element.content_editable(),
        "inherit",
        "Shouldn't have a content_editable"
    );
    element.set_content_editable("true");
    assert_eq!(
        element.content_editable(),
        "true",
        "Should be content_editable"
    );
    assert!(element.is_content_editable(), "Should be content_editable");

    /*TODO doesn't work in Chrome
        // TODO verify case where menu is passed
        match element.context_menu() {
            None => assert!(true, "Shouldn't have a custom menu set"),
            _ => assert!(false, "Shouldn't have a custom menu set")
        };
    */

    // TODO: This test is also broken in Chrome (but not Firefox).
    // assert!(!element.spellcheck(), "Shouldn't be spellchecked");
    element.set_spellcheck(true);
    assert!(element.spellcheck(), "Should be dragspellcheckedgable");

    // TODO verify case where we have an offset_parent
    match element.offset_parent() {
        None => assert!(true, "Shouldn't have an offset_parent set"),
        _ => assert!(false, "Shouldn't have a offset_parent set"),
    };

    // TODO verify when we have offsets
    assert_eq!(element.offset_top(), 0, "Shouldn't have an offset_top yet");
    assert_eq!(
        element.offset_left(),
        0,
        "Shouldn't have an offset_left yet"
    );
    assert_eq!(
        element.offset_width(),
        0,
        "Shouldn't have an offset_width yet"
    );
    assert_eq!(
        element.offset_height(),
        0,
        "Shouldn't have an offset_height yet"
    );
}
