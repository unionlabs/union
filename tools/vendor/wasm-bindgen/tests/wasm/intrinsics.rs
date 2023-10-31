//! Tests that some of our more obscure intrinsics work properly.

use std::convert::TryFrom;
use std::fmt::Debug;

use js_sys::{Array, Object, RangeError, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::wasm_bindgen_test;

// Shorter `JsValue::from(i32)`.
fn int(x: i32) -> JsValue {
    JsValue::from(x)
}

#[wasm_bindgen_test]
fn bitwise() {
    assert_eq!(int(0b1110) & int(0b0111), 0b0110);
    assert_eq!(int(0b1000) | int(0b0010), 0b1010);
    assert_eq!(int(0b1110) ^ int(0b0100), 0b1010);
    assert_eq!(int(0x00ffffff).bit_not(), 0xff000000u32 as i32);

    assert_eq!(int(0b0001) << int(2), 0b0100);
    assert_eq!(int(0b1000) >> int(2), 0b0010);
    assert_eq!(int(-0b1000) >> int(2), -0b0010);
    // Note that bit-wise, this is the same as the above `-0b1000`.
    assert_eq!(
        JsValue::from(0xfffffff8u32).unsigned_shr(&int(2)),
        0x3ffffffe
    );
}

#[wasm_bindgen_test]
fn arithmetic() {
    assert_eq!(-int(12), -12);
    assert_eq!(int(1) + int(2), 3);
    assert_eq!(int(1) - int(2), -1);
    assert_eq!(int(10) / int(5), 2);
    assert_eq!(int(42).checked_div(&int(6)), 7);
    // Note that this doesn't throw for regular numbers, since they just give
    // `NaN` for invalid results.
    assert!(JsValue::bigint_from_str("0")
        .checked_div(&JsValue::bigint_from_str("0"))
        .dyn_into::<RangeError>()
        .is_ok());
    assert_eq!(int(12) * int(34), 408);
    assert_eq!(int(17) % int(10), 7);
    assert_eq!(int(2).pow(&int(8)), 256)
}

#[wasm_bindgen_test]
fn cmp() {
    assert!(int(2).lt(&int(3)));
    assert!(!int(2).lt(&int(2)));

    assert!(int(2).le(&int(3)));
    assert!(int(2).le(&int(2)));

    assert!(int(3).ge(&int(2)));
    assert!(int(3).ge(&int(3)));

    assert!(int(3).gt(&int(2)));
    assert!(!int(3).gt(&int(3)));

    assert!(int(2) == int(2));
    assert!(int(2) != int(3));

    assert!(int(2) != JsValue::from_str("2"));
    assert!(int(2).loose_eq(&JsValue::from_str("2")));
}

#[wasm_bindgen_test]
fn types() {
    assert!(
        Reflect::get(&js_sys::global(), &JsValue::from_str("Number"))
            .unwrap()
            .is_function()
    );
    assert!(JsValue::UNDEFINED.is_undefined());
    assert!(JsValue::NULL.is_null());
    assert!(Object::new().is_object());
    assert!(Array::new().is_array());
    assert!(JsValue::symbol(None).is_symbol());
    assert!(JsValue::from_str("hi").is_string());
    assert!(JsValue::bigint_from_str("5").is_bigint());
    assert_eq!(int(5).js_typeof(), "number");
    assert_eq!(JsValue::bigint_from_str("5").js_typeof(), "bigint");
    assert_eq!(JsValue::NULL.js_typeof(), "object");
}

#[wasm_bindgen_test]
fn misc() {
    assert!(JsValue::from_str("Number").js_in(&js_sys::global()));
    assert!(!JsValue::from_str("frob").js_in(&js_sys::global()));

    assert_eq!(int(5).unchecked_into_f64(), 5.0);
    assert_eq!(JsValue::from_str("5").unchecked_into_f64(), 5.0);

    assert_eq!(f64::try_from(int(5)), Ok(5.0));
    assert_eq!(f64::try_from(JsValue::from_str("5")), Ok(5.0));
    assert!(f64::try_from(JsValue::from_str("hi")).unwrap().is_nan());
    assert!(f64::try_from(JsValue::symbol(None)).is_err());
}

fn debug(x: impl Debug) -> String {
    format!("{x:?}")
}

#[wasm_bindgen_test]
fn debug_string() {
    assert_eq!(debug(int(5)), "JsValue(5)");
    assert_eq!(debug(JsValue::TRUE), "JsValue(true)");
    assert_eq!(debug(JsValue::symbol(None)), "JsValue(Symbol)");
    assert_eq!(debug(JsValue::from_str("hi")), "JsValue(\"hi\")");
}
