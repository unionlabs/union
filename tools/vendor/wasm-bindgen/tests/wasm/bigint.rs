use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/bigint.js")]
extern "C" {
    fn i64_js_identity(a: i64) -> i64;
    fn u64_js_identity(a: u64) -> u64;
    fn js_works();
}

#[wasm_bindgen]
pub fn zero() -> u64 {
    0
}

#[wasm_bindgen]
pub fn one() -> u64 {
    1
}

#[wasm_bindgen]
pub fn neg_one() -> i64 {
    -1
}

#[wasm_bindgen]
pub fn i32_min() -> i64 {
    i32::min_value() as i64
}

#[wasm_bindgen]
pub fn u32_max() -> u64 {
    u32::max_value() as u64
}

#[wasm_bindgen]
pub fn i64_min() -> i64 {
    i64::min_value()
}

#[wasm_bindgen]
pub fn u64_max() -> u64 {
    u64::max_value()
}

#[wasm_bindgen]
pub fn i64_rust_identity(a: i64) -> i64 {
    i64_js_identity(a)
}

#[wasm_bindgen]
pub fn u64_rust_identity(a: u64) -> u64 {
    u64_js_identity(a)
}

#[wasm_bindgen]
pub fn i64_jsvalue_identity(a: i64) -> JsValue {
    JsValue::from(a)
}

#[wasm_bindgen]
pub fn u64_jsvalue_identity(a: u64) -> JsValue {
    JsValue::from(a)
}

#[wasm_bindgen]
pub fn i128_min_jsvalue() -> JsValue {
    JsValue::from(i128::min_value())
}

#[wasm_bindgen]
pub fn u128_max_jsvalue() -> JsValue {
    JsValue::from(u128::max_value())
}

#[wasm_bindgen]
pub fn i64_slice(a: &[i64]) -> Vec<i64> {
    a.to_vec()
}

#[wasm_bindgen]
pub fn u64_slice(a: &[u64]) -> Vec<u64> {
    a.to_vec()
}

#[wasm_bindgen_test]
fn works() {
    js_works();
}

mod try_from_works {
    use super::*;
    use crate::JsValue;
    use core::convert::TryFrom;

    macro_rules! test_type_boundaries {
        ($($ty:ident)*) => {
            $(
                #[wasm_bindgen_test]
                fn $ty() {
                    // Not a bigint.
                    assert!($ty::try_from(JsValue::NULL).is_err());
                    assert!($ty::try_from(JsValue::from_f64(0.0)).is_err());
                    // Within range.
                    assert_eq!($ty::try_from(JsValue::from($ty::MIN)), Ok($ty::MIN));
                    // Too small.
                    assert!($ty::try_from(JsValue::from($ty::MIN) - JsValue::from(1_i64)).is_err());
                    // Within range.
                    assert_eq!($ty::try_from(JsValue::from($ty::MAX)), Ok($ty::MAX));
                    // Too large.
                    assert!($ty::try_from(JsValue::from($ty::MAX) + JsValue::from(1_i64)).is_err());
                }
            )*
        };
    }

    test_type_boundaries!(i64 u64 i128 u128);
}
