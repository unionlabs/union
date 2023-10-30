use std::f64::{INFINITY, NAN};

use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/Number.js")]
extern "C" {
    fn const_epsilon() -> f64;
    fn const_max_safe_integer() -> f64;
    fn const_max_value() -> f64;
    fn const_min_safe_integer() -> f64;
    fn const_min_value() -> f64;
    fn const_negative_infinity() -> f64;
    fn const_positive_infinity() -> f64;
}

#[wasm_bindgen_test]
fn is_finite() {
    assert!(Number::is_finite(&42.into()));
    assert!(Number::is_finite(&42.1.into()));
    assert!(!Number::is_finite(&"42".into()));
    assert!(!Number::is_finite(&NAN.into()));
    assert!(!Number::is_finite(&INFINITY.into()));
}

#[wasm_bindgen_test]
fn is_integer() {
    assert!(Number::is_integer(&42.into()));
    assert!(!Number::is_integer(&42.1.into()));
}

#[wasm_bindgen_test]
fn is_nan() {
    assert!(Number::is_nan(&NAN.into()));

    assert!(!Number::is_nan(&JsValue::TRUE));
    assert!(!Number::is_nan(&JsValue::NULL));
    assert!(!Number::is_nan(&37.into()));
    assert!(!Number::is_nan(&"37".into()));
    assert!(!Number::is_nan(&"37.37".into()));
    assert!(!Number::is_nan(&"".into()));
    assert!(!Number::is_nan(&" ".into()));

    // These would all return true with the global isNaN()
    assert!(!Number::is_nan(&"NaN".into()));
    assert!(!Number::is_nan(&JsValue::UNDEFINED));
    assert!(!Number::is_nan(&"blabla".into()));
}

#[wasm_bindgen_test]
fn is_safe_integer() {
    assert_eq!(Number::is_safe_integer(&42.into()), true);
    assert_eq!(
        Number::is_safe_integer(&(Math::pow(2., 53.) - 1.).into()),
        true
    );
    assert_eq!(Number::is_safe_integer(&Math::pow(2., 53.).into()), false);
    assert_eq!(Number::is_safe_integer(&"42".into()), false);
    assert_eq!(Number::is_safe_integer(&42.1.into()), false);
    assert_eq!(Number::is_safe_integer(&NAN.into()), false);
    assert_eq!(Number::is_safe_integer(&INFINITY.into()), false);
}

#[allow(deprecated)]
#[wasm_bindgen_test]
fn new() {
    let n = Number::new(&JsValue::from(42));
    let v = JsValue::from(n);
    assert!(v.is_object());
    assert_eq!(Number::from(v).value_of(), 42.);
}

#[wasm_bindgen_test]
fn parse_int_float() {
    assert_eq!(Number::parse_int("42", 10), 42.);
    assert_eq!(Number::parse_int("42", 16), 66.); // 0x42 == 66
    assert!(Number::parse_int("invalid int", 10).is_nan());

    assert_eq!(Number::parse_float("123456.789"), 123456.789);
    assert!(Number::parse_float("invalid float").is_nan());
}

#[wasm_bindgen_test]
fn to_locale_string() {
    let number = Number::from(1234.45);
    assert_eq!(number.to_locale_string("en-US"), "1,234.45");
    // TODO: these tests seems to be system dependent, disable for now
    // assert_eq!(wasm.to_locale_string(number, "de-DE"), "1,234.45");
    // assert_eq!(wasm.to_locale_string(number, "zh-Hans-CN-u-nu-hanidec"), "1,234.45");
}

#[wasm_bindgen_test]
fn to_precision() {
    assert_eq!(Number::from(0.1).to_precision(3).unwrap(), "0.100");
    assert!(Number::from(10).to_precision(101).is_err());
}

#[wasm_bindgen_test]
fn to_string() {
    assert_eq!(Number::from(42).to_string(10).unwrap(), "42");
    assert_eq!(Number::from(233).to_string(16).unwrap(), "e9");
    assert!(Number::from(100).to_string(100).is_err());
}

#[wasm_bindgen_test]
fn value_of() {
    assert_eq!(Number::from(42).value_of(), 42.);
}

#[wasm_bindgen_test]
fn to_fixed() {
    assert_eq!(Number::from(123.456).to_fixed(2).unwrap(), "123.46");
    assert!(Number::from(10).to_fixed(101).is_err());
}

#[wasm_bindgen_test]
fn to_exponential() {
    assert_eq!(Number::from(123456).to_exponential(2).unwrap(), "1.23e+5");
    assert!(Number::from(10).to_exponential(101).is_err());
}

#[allow(deprecated)]
#[wasm_bindgen_test]
fn number_inheritance() {
    let n = Number::new(&JsValue::from(42));
    assert!(n.is_instance_of::<Number>());
    assert!(n.is_instance_of::<Object>());
    let _: &Object = n.as_ref();
}

#[wasm_bindgen_test]
fn consts() {
    assert_eq!(const_epsilon(), Number::EPSILON, "EPSILON");
    assert_eq!(
        const_max_safe_integer(),
        Number::MAX_SAFE_INTEGER,
        "MAX_SAFE_INTEGER"
    );
    assert_eq!(const_max_value(), Number::MAX_VALUE, "MAX_VALUE");
    assert_eq!(
        const_min_safe_integer(),
        Number::MIN_SAFE_INTEGER,
        "MIN_SAFE_INTEGER"
    );
    assert_eq!(const_min_value(), Number::MIN_VALUE, "MIN_VALUE");
    assert_eq!(
        const_negative_infinity(),
        Number::NEGATIVE_INFINITY,
        "NEGATIVE_INFINITY"
    );
    assert_eq!(
        const_positive_infinity(),
        Number::POSITIVE_INFINITY,
        "POSITIVE_INFINITY"
    );
}
