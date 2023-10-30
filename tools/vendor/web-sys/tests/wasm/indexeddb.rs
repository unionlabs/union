use wasm_bindgen_test::*;
use web_sys;

#[wasm_bindgen_test]
fn accessor_works() {
    let window = web_sys::window().unwrap();
    assert!(window.indexed_db().unwrap().is_some());
}
