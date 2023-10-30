use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{History, ScrollRestoration};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = history, js_namespace = window)]
    static HISTORY: History;
}

#[wasm_bindgen_test]
fn history() {
    HISTORY
        .set_scroll_restoration(ScrollRestoration::Manual)
        .expect("failure to set scroll restoration");
    assert_eq!(
        HISTORY.scroll_restoration().unwrap(),
        ScrollRestoration::Manual
    );

    HISTORY
        .set_scroll_restoration(ScrollRestoration::Auto)
        .expect("failure to set scroll restoration");
    assert_eq!(
        HISTORY.scroll_restoration().unwrap(),
        ScrollRestoration::Auto
    );
}
