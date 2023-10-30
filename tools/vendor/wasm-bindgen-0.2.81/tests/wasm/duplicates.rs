use wasm_bindgen_test::*;

pub mod same_function_different_locations_a {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "tests/wasm/duplicates_a.js")]
    extern "C" {
        pub fn foo();
        pub static bar: JsValue;
    }
}

pub mod same_function_different_locations_b {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "tests/wasm/duplicates_a.js")]
    extern "C" {
        pub fn foo();
        pub static bar: JsValue;
    }
}

#[wasm_bindgen_test]
fn same_function_different_locations() {
    same_function_different_locations_a::foo();
    same_function_different_locations_b::foo();
    assert_eq!(*same_function_different_locations_a::bar, 3);
    assert_eq!(*same_function_different_locations_a::bar, 3);
}

pub mod same_function_different_modules_a {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "tests/wasm/duplicates_b.js")]
    extern "C" {
        pub fn foo() -> bool;
        pub static bar: JsValue;
    }
}

pub mod same_function_different_modules_b {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "tests/wasm/duplicates_c.js")]
    extern "C" {
        pub fn foo() -> bool;
        pub static bar: JsValue;
    }
}

#[wasm_bindgen_test]
fn same_function_different_modules() {
    assert!(same_function_different_modules_a::foo());
    assert!(!same_function_different_modules_b::foo());
    assert_eq!(*same_function_different_modules_a::bar, 4);
    assert_eq!(*same_function_different_modules_b::bar, 5);
}
