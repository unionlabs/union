use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/js_keywords.js")]
extern "C" {
    fn js_keywords_compile();
    fn test_keyword_1_as_fn_name(x: u8) -> u8;
    fn test_keyword_2_as_fn_name(x: u8, y: u8) -> bool;
    fn test_keyword_as_fn_arg(x: u8) -> u8;
}

#[wasm_bindgen]
pub fn throw(class: u8) -> u8 {
    class
}

#[wasm_bindgen(js_name = class)]
pub fn fn_parsed_to_keyword(instanceof: u8, catch: u8) -> bool {
    instanceof > catch
}

#[wasm_bindgen(js_name = classy)]
pub fn arg_is_keyword(class: u8) -> u8 {
    class
}

#[wasm_bindgen]
pub struct Class {
    name: String,
}
#[wasm_bindgen]
impl Class {
    #[wasm_bindgen(constructor)]
    pub fn new(void: String) -> Self {
        Class { name: void }
    }
    pub fn instanceof(&self, class: String) -> String {
        format!("{} is instance of {}", self.name.clone(), class)
    }
    #[wasm_bindgen(getter)]
    pub fn catch(&self) -> String {
        self.name.clone()
    }
    pub fn void(void: String) -> String {
        void
    }
}

#[wasm_bindgen_test]
fn compile() {
    js_keywords_compile();
    assert_eq!(test_keyword_1_as_fn_name(1), 1);
    assert!(!test_keyword_2_as_fn_name(1, 2));
    assert_eq!(test_keyword_as_fn_arg(1), 1);
}
