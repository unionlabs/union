use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "/tests/wasm/link_to.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn read_file(url: &str) -> Result<String, JsValue>;
}

#[wasm_bindgen_test]
fn test_module() {
    let link = wasm_bindgen::link_to!(module = "/tests/wasm/linked_module.js");
    assert_eq!(read_file(&link).unwrap(), "// linked module\n");
}

#[wasm_bindgen_test]
fn test_raw_module() {
    let link = wasm_bindgen::link_to!(raw_module = "./not-found.js");
    assert!(read_file(&link).is_err());
}

#[wasm_bindgen_test]
fn test_inline_js() {
    // Test two invocations to ensure that snippet indices from different
    // Program structs are offset correctly.
    let link1 = wasm_bindgen::link_to!(inline_js = "// inline js 1\n");
    let link2 = wasm_bindgen::link_to!(inline_js = "// inline js 2\n");
    assert_eq!(read_file(&link1).unwrap(), "// inline js 1\n");
    assert_eq!(read_file(&link2).unwrap(), "// inline js 2\n");
}
