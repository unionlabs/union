use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/validate_prt.js")]
extern "C" {
    fn js_works();
}

#[wasm_bindgen]
pub struct Fruit {
    name: String,
}

#[wasm_bindgen]
impl Fruit {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(constructor)]
    pub fn new(name: String) -> Self {
        Fruit { name }
    }

    pub fn rot(self) {
        drop(self);
    }

    #[wasm_bindgen(getter)]
    pub fn prop(self) -> u32 {
        0
    }

    #[wasm_bindgen(setter)]
    pub fn set_prop(self, _val: u32) {}
}

#[wasm_bindgen]
pub fn eat(_: Fruit) {}

#[wasm_bindgen_test]
fn works() {
    js_works();
}
