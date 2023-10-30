use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/comments.js")]
extern "C" {
    fn assert_comments_exist();
}

/// annotated function ✔️ " \ ' {
#[wasm_bindgen]
pub fn annotated() -> String {
    String::new()
}

/// annotated struct type
#[wasm_bindgen]
pub struct Annotated {
    a: String,
    /// annotated struct field b
    pub b: u32,
    /// annotated struct field c
    #[wasm_bindgen(readonly)]
    pub c: u32,
    d: u32,
}

#[wasm_bindgen]
impl Annotated {
    /// annotated struct constructor
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            a: String::new(),
            b: 0,
            c: 0,
            d: 0,
        }
    }

    /// annotated struct method
    pub fn get_a(&self) -> String {
        self.a.clone()
    }

    /// annotated struct getter
    #[wasm_bindgen(getter)]
    pub fn d(&self) -> u32 {
        self.d
    }

    /// annotated struct setter
    #[wasm_bindgen(setter)]
    pub fn set_d(&mut self, value: u32) {
        self.d = value
    }

    /// annotated struct static method
    pub fn static_method() {}
}

/// annotated enum type
#[wasm_bindgen]
pub enum AnnotatedEnum {
    /// annotated enum variant 1
    Variant1,
    /// annotated enum variant 2
    Variant2,
}

#[wasm_bindgen_test]
fn works() {
    assert_comments_exist();
}
