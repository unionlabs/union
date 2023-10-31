use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/optional_primitives.js")]
extern "C" {
    fn optional_i32_js_identity(a: Option<i32>) -> Option<i32>;
    fn optional_u32_js_identity(a: Option<u32>) -> Option<u32>;
    fn optional_isize_js_identity(a: Option<isize>) -> Option<isize>;
    fn optional_usize_js_identity(a: Option<usize>) -> Option<usize>;
    fn optional_f32_js_identity(a: Option<f32>) -> Option<f32>;
    fn optional_f64_js_identity(a: Option<f64>) -> Option<f64>;
    fn optional_i8_js_identity(a: Option<i8>) -> Option<i8>;
    fn optional_u8_js_identity(a: Option<u8>) -> Option<u8>;
    fn optional_i16_js_identity(a: Option<i16>) -> Option<i16>;
    fn optional_u16_js_identity(a: Option<u16>) -> Option<u16>;
    fn optional_i64_js_identity(a: Option<i64>) -> Option<i64>;
    fn optional_u64_js_identity(a: Option<u64>) -> Option<u64>;
    fn optional_bool_js_identity(a: Option<bool>) -> Option<bool>;
    fn optional_char_js_identity(a: Option<char>) -> Option<char>;

    fn js_works();
}

#[wasm_bindgen]
pub fn optional_i32_none() -> Option<i32> {
    None
}

#[wasm_bindgen]
pub fn optional_i32_zero() -> Option<i32> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_i32_one() -> Option<i32> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_i32_neg_one() -> Option<i32> {
    Some(-1)
}

#[wasm_bindgen]
pub fn optional_i32_min() -> Option<i32> {
    Some(i32::min_value())
}

#[wasm_bindgen]
pub fn optional_i32_max() -> Option<i32> {
    Some(i32::max_value())
}

#[wasm_bindgen]
pub fn optional_i32_identity(a: Option<i32>) -> Option<i32> {
    optional_i32_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_u32_none() -> Option<u32> {
    None
}

#[wasm_bindgen]
pub fn optional_u32_zero() -> Option<u32> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_u32_one() -> Option<u32> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_u32_min() -> Option<u32> {
    Some(u32::min_value())
}

#[wasm_bindgen]
pub fn optional_u32_max() -> Option<u32> {
    Some(u32::max_value())
}

#[wasm_bindgen]
pub fn optional_u32_identity(a: Option<u32>) -> Option<u32> {
    optional_u32_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_isize_none() -> Option<isize> {
    None
}

#[wasm_bindgen]
pub fn optional_isize_zero() -> Option<isize> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_isize_one() -> Option<isize> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_isize_neg_one() -> Option<isize> {
    Some(-1)
}

#[wasm_bindgen]
pub fn optional_isize_min() -> Option<isize> {
    Some(isize::min_value())
}

#[wasm_bindgen]
pub fn optional_isize_max() -> Option<isize> {
    Some(isize::max_value())
}

#[wasm_bindgen]
pub fn optional_isize_identity(a: Option<isize>) -> Option<isize> {
    optional_isize_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_usize_none() -> Option<usize> {
    None
}

#[wasm_bindgen]
pub fn optional_usize_zero() -> Option<usize> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_usize_one() -> Option<usize> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_usize_min() -> Option<usize> {
    Some(usize::min_value())
}

#[wasm_bindgen]
pub fn optional_usize_max() -> Option<usize> {
    Some(usize::max_value())
}

#[wasm_bindgen]
pub fn optional_usize_identity(a: Option<usize>) -> Option<usize> {
    optional_usize_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_f32_none() -> Option<f32> {
    None
}

#[wasm_bindgen]
pub fn optional_f32_zero() -> Option<f32> {
    Some(0f32)
}

#[wasm_bindgen]
pub fn optional_f32_one() -> Option<f32> {
    Some(1f32)
}

#[wasm_bindgen]
pub fn optional_f32_neg_one() -> Option<f32> {
    Some(-1f32)
}

#[wasm_bindgen]
pub fn optional_f32_identity(a: Option<f32>) -> Option<f32> {
    optional_f32_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_f64_none() -> Option<f64> {
    None
}

#[wasm_bindgen]
pub fn optional_f64_zero() -> Option<f64> {
    Some(0f64)
}

#[wasm_bindgen]
pub fn optional_f64_one() -> Option<f64> {
    Some(1f64)
}

#[wasm_bindgen]
pub fn optional_f64_neg_one() -> Option<f64> {
    Some(-1f64)
}

#[wasm_bindgen]
pub fn optional_f64_identity(a: Option<f64>) -> Option<f64> {
    optional_f64_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_i8_none() -> Option<i8> {
    None
}

#[wasm_bindgen]
pub fn optional_i8_zero() -> Option<i8> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_i8_one() -> Option<i8> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_i8_neg_one() -> Option<i8> {
    Some(-1)
}

#[wasm_bindgen]
pub fn optional_i8_min() -> Option<i8> {
    Some(i8::min_value())
}

#[wasm_bindgen]
pub fn optional_i8_max() -> Option<i8> {
    Some(i8::max_value())
}

#[wasm_bindgen]
pub fn optional_i8_identity(a: Option<i8>) -> Option<i8> {
    optional_i8_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_u8_none() -> Option<u8> {
    None
}

#[wasm_bindgen]
pub fn optional_u8_zero() -> Option<u8> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_u8_one() -> Option<u8> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_u8_min() -> Option<u8> {
    Some(u8::min_value())
}

#[wasm_bindgen]
pub fn optional_u8_max() -> Option<u8> {
    Some(u8::max_value())
}

#[wasm_bindgen]
pub fn optional_u8_identity(a: Option<u8>) -> Option<u8> {
    optional_u8_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_i16_none() -> Option<i16> {
    None
}

#[wasm_bindgen]
pub fn optional_i16_zero() -> Option<i16> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_i16_one() -> Option<i16> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_i16_neg_one() -> Option<i16> {
    Some(-1)
}

#[wasm_bindgen]
pub fn optional_i16_min() -> Option<i16> {
    Some(i16::min_value())
}

#[wasm_bindgen]
pub fn optional_i16_max() -> Option<i16> {
    Some(i16::max_value())
}

#[wasm_bindgen]
pub fn optional_i16_identity(a: Option<i16>) -> Option<i16> {
    optional_i16_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_u16_none() -> Option<u16> {
    None
}

#[wasm_bindgen]
pub fn optional_u16_zero() -> Option<u16> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_u16_one() -> Option<u16> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_u16_min() -> Option<u16> {
    Some(u16::min_value())
}

#[wasm_bindgen]
pub fn optional_u16_max() -> Option<u16> {
    Some(u16::max_value())
}

#[wasm_bindgen]
pub fn optional_u16_identity(a: Option<u16>) -> Option<u16> {
    optional_u16_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_i64_none() -> Option<i64> {
    None
}

#[wasm_bindgen]
pub fn optional_i64_zero() -> Option<i64> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_i64_one() -> Option<i64> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_i64_neg_one() -> Option<i64> {
    Some(-1)
}

#[wasm_bindgen]
pub fn optional_i64_min() -> Option<i64> {
    Some(i64::min_value())
}

#[wasm_bindgen]
pub fn optional_i64_max() -> Option<i64> {
    Some(i64::max_value())
}

#[wasm_bindgen]
pub fn optional_i64_identity(a: Option<i64>) -> Option<i64> {
    optional_i64_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_u64_none() -> Option<u64> {
    None
}

#[wasm_bindgen]
pub fn optional_u64_zero() -> Option<u64> {
    Some(0)
}

#[wasm_bindgen]
pub fn optional_u64_one() -> Option<u64> {
    Some(1)
}

#[wasm_bindgen]
pub fn optional_u64_min() -> Option<u64> {
    Some(u64::min_value())
}

#[wasm_bindgen]
pub fn optional_u64_max() -> Option<u64> {
    Some(u64::max_value())
}

#[wasm_bindgen]
pub fn optional_u64_identity(a: Option<u64>) -> Option<u64> {
    optional_u64_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_bool_none() -> Option<bool> {
    None
}

#[wasm_bindgen]
pub fn optional_bool_false() -> Option<bool> {
    Some(false)
}

#[wasm_bindgen]
pub fn optional_bool_true() -> Option<bool> {
    Some(true)
}

#[wasm_bindgen]
pub fn optional_bool_identity(a: Option<bool>) -> Option<bool> {
    optional_bool_js_identity(a)
}

#[wasm_bindgen]
pub fn optional_char_none() -> Option<char> {
    None
}

#[wasm_bindgen]
pub fn optional_char_letter() -> Option<char> {
    Some('a')
}

#[wasm_bindgen]
pub fn optional_char_face() -> Option<char> {
    Some('😀')
}

#[wasm_bindgen]
pub fn optional_char_identity(a: Option<char>) -> Option<char> {
    optional_char_js_identity(a)
}

#[wasm_bindgen_test]
fn works() {
    js_works();
}
