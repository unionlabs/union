use std::f64::{INFINITY, NAN};

use js_sys::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn test_decode_uri() {
    let x = decode_uri("https://mozilla.org/?x=%D1%88%D0%B5%D0%BB%D0%BB%D1%8B")
        .ok()
        .expect("should decode URI OK");
    assert_eq!(String::from(x), "https://mozilla.org/?x=шеллы");

    assert!(decode_uri("%E0%A4%A").is_err());
}

#[wasm_bindgen_test]
fn test_decode_uri_component() {
    let x = decode_uri_component("%3Fx%3Dtest")
        .ok()
        .expect("should decode URI OK");
    assert_eq!(String::from(x), "?x=test");

    assert!(decode_uri_component("%E0%A4%A").is_err());
}

#[wasm_bindgen_test]
fn test_encode_uri() {
    let x = encode_uri("ABC abc 123");
    assert_eq!(String::from(x), "ABC%20abc%20123");
}

#[wasm_bindgen_test]
fn test_encode_uri_component() {
    let x = encode_uri_component("?x=шеллы");
    assert_eq!(String::from(x), "%3Fx%3D%D1%88%D0%B5%D0%BB%D0%BB%D1%8B");
}

#[wasm_bindgen_test]
fn test_eval() {
    let x = eval("42").ok().expect("should eval OK");
    assert_eq!(x.as_f64().unwrap(), 42.0);

    let err = eval("(function () { throw 42; }())")
        .err()
        .expect("eval should throw");
    assert_eq!(err.as_f64().unwrap(), 42.0);
}

#[wasm_bindgen_test]
fn test_is_finite() {
    assert!(is_finite(&42.into()));
    assert!(is_finite(&42.1.into()));
    assert!(is_finite(&"42".into()));
    assert!(!is_finite(&NAN.into()));
    assert!(!is_finite(&INFINITY.into()));
}

#[wasm_bindgen_test]
fn test_parse_int_float() {
    let i = parse_int("42", 10);
    assert_eq!(i as i64, 42);

    let i = parse_int("42", 16);
    assert_eq!(i as i64, 66); // 0x42 == 66

    let i = parse_int("invalid int", 10);
    assert!(i.is_nan());

    let f = parse_float("123456.789");
    assert_eq!(f, 123456.789);

    let f = parse_float("invalid float");
    assert!(f.is_nan());
}

#[wasm_bindgen_test]
fn test_escape() {
    assert_eq!(String::from(escape("test")), "test");
    assert_eq!(String::from(escape("äöü")), "%E4%F6%FC");
    assert_eq!(String::from(escape("ć")), "%u0107");
    assert_eq!(String::from(escape("@*_+-./")), "@*_+-./");
}

#[wasm_bindgen_test]
fn test_unescape() {
    assert_eq!(String::from(unescape("abc123")), "abc123");
    assert_eq!(String::from(unescape("%E4%F6%FC")), "äöü");
    assert_eq!(String::from(unescape("%u0107")), "ć");
    assert_eq!(String::from(unescape("@*_+-./")), "@*_+-./");
}
