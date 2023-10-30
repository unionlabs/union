use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

// This test is in the headless suite so that we can test the `externref` table
// implementation of `externref_heap_live_count` (as opposed to the JS `heap`
// implementation) in Firefox.
#[wasm_bindgen_test]
fn test_externref_heap_live_count() {
    let initial = wasm_bindgen::externref_heap_live_count();

    let after_alloc = {
        let _vals: Vec<_> = (0..10).map(JsValue::from).collect();
        wasm_bindgen::externref_heap_live_count()
    };

    let after_dealloc = wasm_bindgen::externref_heap_live_count();

    assert_eq!(initial, after_dealloc);
    assert_eq!(initial + 10, after_alloc);
}
