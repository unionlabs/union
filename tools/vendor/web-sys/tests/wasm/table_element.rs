use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{HtmlTableCaptionElement, HtmlTableElement, HtmlTableSectionElement};

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_table() -> HtmlTableElement;
    fn new_caption() -> HtmlTableCaptionElement;
    fn new_thead() -> HtmlTableSectionElement;
    fn new_tfoot() -> HtmlTableSectionElement;
}

#[wasm_bindgen_test]
fn test_table_element() {
    let table = new_table();
    assert!(
        table.caption().is_none(),
        "New table element should have no caption element."
    );

    table.create_caption();
    assert!(
        table.caption().is_some(),
        "Table element should have caption element after create caption."
    );

    table.delete_caption();
    assert!(
        table.caption().is_none(),
        "Table element should have no caption element after delete caption."
    );

    table.set_caption(Some(&new_caption()));
    assert!(
        table.caption().is_some(),
        "Table element should have caption element after set."
    );

    assert!(
        table.t_head().is_none(),
        "New table element should have no thead element."
    );

    table.create_t_head();
    assert!(
        table.t_head().is_some(),
        "Table element should have thead element after create thead."
    );

    table.delete_t_head();
    assert!(
        table.t_head().is_none(),
        "Table element should have no thead element after delete thead."
    );

    table.set_t_head(Some(&new_thead()));
    assert!(
        table.t_head().is_some(),
        "Table element should have thead element after set."
    );

    assert!(
        table.t_foot().is_none(),
        "New table element should have no tfoot element."
    );

    table.create_t_foot();
    assert!(
        table.t_foot().is_some(),
        "Table element should have tfoot element after create tfoot."
    );

    table.delete_t_foot();
    assert!(
        table.t_foot().is_none(),
        "Table element should have no tfoot element after delete tfoot."
    );

    table.set_t_foot(Some(&new_tfoot()));
    assert!(
        table.t_foot().is_some(),
        "Table element should have tfoot element after set."
    );

    assert!(
        table.t_bodies().length() == 0,
        "New table element should have no tbody element."
    );

    table.create_t_body();
    assert!(
        table.t_bodies().length() == 1,
        "Table element should have tbody element after create tbody."
    );

    assert!(
        table.rows().length() == 0,
        "New table element should have no rows."
    );

    table
        .insert_row_with_index(0)
        .expect("Failed to insert row at index 0");
    assert!(
        table.rows().length() == 1,
        "Table element should have rows after insert row."
    );

    table
        .delete_row(0)
        .expect("Failed to delete row at index 0");
    assert!(
        table.rows().length() == 0,
        "Table element should have no rows after delete row."
    );

    table.set_align("left");
    assert_eq!(
        table.align(),
        "left",
        "Table element should have an align property of 'left'"
    );

    table.set_border("10");
    assert_eq!(
        table.border(),
        "10",
        "Table element should have a border property of '10'"
    );

    table.set_frame("above");
    assert_eq!(
        table.frame(),
        "above",
        "Table element should have an frame property of 'above'"
    );

    table.set_rules("none");
    assert_eq!(
        table.rules(),
        "none",
        "Table element should have an rules property of 'none'"
    );

    table.set_summary("summary");
    assert_eq!(
        table.summary(),
        "summary",
        "Table element should have an summary property of 'summary'"
    );

    table.set_width("1000");
    assert_eq!(
        table.width(),
        "1000",
        "Table element should have a width property of '1000'"
    );

    table.set_bg_color("#ffffff");
    assert_eq!(
        table.bg_color(),
        "#ffffff",
        "Table element should have an bgColor property of '#ffffff'"
    );

    table.set_cell_padding("1");
    assert_eq!(
        table.cell_padding(),
        "1",
        "Table element should have an cellPadding property of '1'"
    );

    table.set_cell_spacing("1");
    assert_eq!(
        table.cell_spacing(),
        "1",
        "Table element should have an cellSpacing property of '1'"
    );
}
