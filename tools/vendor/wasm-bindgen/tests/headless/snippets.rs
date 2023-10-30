use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "/tests/headless/snippets1.js")]
extern "C" {
    fn get_two() -> u32;
    #[wasm_bindgen(js_name = get_stateful)]
    fn get_stateful1() -> u32;
}

#[wasm_bindgen(module = "/tests/headless/snippets1.js")]
extern "C" {
    #[wasm_bindgen(js_name = get_stateful)]
    fn get_stateful2() -> u32;
}

#[wasm_bindgen_test]
fn test_get_two() {
    assert_eq!(get_two(), 2);
}

#[wasm_bindgen_test]
fn stateful_deduplicated() {
    assert_eq!(get_stateful1(), 1);
    assert_eq!(get_stateful2(), 2);
    assert_eq!(get_stateful1(), 3);
    assert_eq!(get_stateful2(), 4);
}

#[wasm_bindgen(inline_js = "export function get_three() { return 3; }")]
extern "C" {
    fn get_three() -> u32;
}

#[wasm_bindgen_test]
fn test_get_three() {
    assert_eq!(get_three(), 3);
}

#[wasm_bindgen(inline_js = "let a = 0; export function get() { a += 1; return a; }")]
extern "C" {
    #[wasm_bindgen(js_name = get)]
    fn duplicate1() -> u32;
}

#[wasm_bindgen(inline_js = "let a = 0; export function get() { a += 1; return a; }")]
extern "C" {
    #[wasm_bindgen(js_name = get)]
    fn duplicate2() -> u32;
}

#[wasm_bindgen_test]
fn duplicate_inline_not_unified() {
    assert_eq!(duplicate1(), 1);
    assert_eq!(duplicate2(), 1);
    assert_eq!(duplicate1(), 2);
    assert_eq!(duplicate2(), 2);
}
