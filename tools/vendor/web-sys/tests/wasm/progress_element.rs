use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::HtmlProgressElement;

#[wasm_bindgen(module = "/tests/wasm/element.js")]
extern "C" {
    fn new_progress() -> HtmlProgressElement;
}

#[wasm_bindgen_test]
fn test_progress_element() {
    let progress = new_progress();
    progress.set_max(150.5);
    assert_eq!(
        progress.max(),
        150.5,
        "Maximum progress value should be 150.5."
    );

    progress.set_value(22.);
    assert_eq!(progress.value(), 22., "Progress value should be 22 units.");
    assert_eq!(
        progress.position(),
        (22. / 150.5),
        "Progress position should be 22 divided by the max possible value."
    );

    assert!(
        progress.labels().length() == 0,
        "Our simple progress bar shouldn't be associated with any labels."
    );
}
