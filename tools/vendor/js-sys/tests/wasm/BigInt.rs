use js_sys::BigInt;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::wasm_bindgen_test;

/// `assert_eq!`, but the arguments are converted to `JsValue`s.
#[track_caller]
fn assert_jsvalue_eq(a: impl Into<JsValue>, b: impl Into<JsValue>) {
    assert_eq!(a.into(), b.into());
}

#[wasm_bindgen_test]
fn from() {
    // Test that all the `From` impls work properly.
    assert_jsvalue_eq(BigInt::from(1u8), 1u64);
    assert_jsvalue_eq(BigInt::from(1u16), 1u64);
    assert_jsvalue_eq(BigInt::from(1u32), 1u64);
    assert_jsvalue_eq(BigInt::from(1u64), 1u64);
    assert_jsvalue_eq(BigInt::from(1u128), 1u64);
    assert_jsvalue_eq(BigInt::from(1usize), 1u64);
    assert_jsvalue_eq(BigInt::from(-3i8), -3i64);
    assert_jsvalue_eq(BigInt::from(-3i16), -3i64);
    assert_jsvalue_eq(BigInt::from(-3i32), -3i64);
    assert_jsvalue_eq(BigInt::from(-3i64), -3i64);
    assert_jsvalue_eq(BigInt::from(-3i128), -3i64);
    assert_jsvalue_eq(BigInt::from(-3isize), -3i64);
}

#[wasm_bindgen_test]
fn eq() {
    // Test that all the `Eq` impls work properly.
    assert_eq!(BigInt::from(1u64), 1u8);
    assert_eq!(BigInt::from(1u64), 1u16);
    assert_eq!(BigInt::from(1u64), 1u32);
    assert_eq!(BigInt::from(1u64), 1u64);
    assert_eq!(BigInt::from(1u64), 1u128);
    assert_eq!(BigInt::from(1u64), 1usize);
    assert_eq!(BigInt::from(-3i64), -3i8);
    assert_eq!(BigInt::from(-3i64), -3i16);
    assert_eq!(BigInt::from(-3i64), -3i32);
    assert_eq!(BigInt::from(-3i64), -3i64);
    assert_eq!(BigInt::from(-3i64), -3i128);
    assert_eq!(BigInt::from(-3i64), -3isize);
}
