use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen]
pub struct OwnedValue {
    pub n: f64,
}

#[wasm_bindgen]
impl OwnedValue {
    #[wasm_bindgen(constructor)]
    pub fn new(n: f64) -> Self {
        Self { n }
    }

    #[allow(clippy::should_implement_trait)] // traits unsupported by wasm_bindgen
    pub fn add(self, other: OwnedValue) -> Self {
        Self {
            n: self.n + other.n,
        }
    }

    pub fn n(self) -> f64 {
        self.n
    }
}

#[wasm_bindgen(module = "tests/wasm/owned.js")]
extern "C" {
    fn create_garbage();
}

#[wasm_bindgen_test]
fn test_create_garbage() {
    create_garbage()
}
