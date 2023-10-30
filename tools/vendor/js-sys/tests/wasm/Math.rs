use std::f64::consts::PI;
use std::f64::{NAN, NEG_INFINITY};

use js_sys::*;
use wasm_bindgen_test::*;

macro_rules! assert_eq {
    ($a:expr, $b:expr) => {{
        let (a, b) = (&$a, &$b);
        if f64::is_infinite(*a) && f64::is_infinite(*b) {
            assert!(a == b);
        } else {
            assert!(
                (*a - *b).abs() < 1.0e-6,
                "not approximately equal {:?} ?= {:?}",
                a,
                b
            );
        }
    }};
}

#[wasm_bindgen_test]
fn abs() {
    assert_eq!(Math::abs(-32.), 32.);
    assert_eq!(Math::abs(-32.), 32.);
    assert_eq!(Math::abs(-4.7), 4.7);
}

#[wasm_bindgen_test]
fn acos() {
    assert_eq!(Math::acos(-1.), PI);
    assert_eq!(Math::acos(0.5), 1.0471975511965979);
    assert!(Math::acos(2.).is_nan());
}

#[wasm_bindgen_test]
fn acosh() {
    assert_eq!(Math::acosh(1.), 0.);
    assert_eq!(Math::acosh(2.), 2.0f64.acosh());
    assert!(Math::acosh(0.5).is_nan());
}

#[wasm_bindgen_test]
fn asin() {
    assert_eq!(Math::asin(1.), 1.0f64.asin());
    assert_eq!(Math::asin(0.5), 0.5f64.asin());
    assert!(Math::asin(2.).is_nan());
}

#[wasm_bindgen_test]
fn asinh() {
    assert_eq!(Math::asinh(1.0), 1f64.asinh());
    assert_eq!(Math::asinh(0.5), 0.5f64.asinh());
}

#[wasm_bindgen_test]
fn atan() {
    assert_eq!(Math::atan(1.0), 1f64.atan());
    assert_eq!(Math::atan(0.5), 0.5f64.atan());
}

#[wasm_bindgen_test]
fn atan2() {
    assert_eq!(Math::atan2(1.0, 2.0), 1f64.atan2(2.));
    assert_eq!(Math::atan2(0.7, 3.8), 0.7f64.atan2(3.8f64));
}

#[wasm_bindgen_test]
fn atanh() {
    assert_eq!(Math::atanh(1.), 1f64.atanh());
    assert_eq!(Math::atanh(0.5), 0.5f64.atanh());
    assert!(Math::atanh(2.).is_nan());
}

#[wasm_bindgen_test]
fn cbrt() {
    assert_eq!(Math::cbrt(27.), 3.);
    assert_eq!(Math::cbrt(12.3), 12.3f64.cbrt());
}

#[wasm_bindgen_test]
fn ceil() {
    assert_eq!(Math::ceil(1.1), 2.);
    assert_eq!(Math::ceil(-1.1), -1.);
}

#[wasm_bindgen_test]
fn clz32() {
    assert!(Math::clz32(1) == 31);
    assert!(Math::clz32(1000) == 22);
}

#[wasm_bindgen_test]
fn cos() {
    assert_eq!(Math::cos(0.0), 1.);
    assert_eq!(Math::cos(1.5), 1.5f64.cos());
}

#[wasm_bindgen_test]
fn cosh() {
    assert_eq!(Math::cosh(0.), 1.);
    assert_eq!(Math::cosh(2.), 3.7621956910836314);
}

#[wasm_bindgen_test]
fn exp() {
    assert_eq!(Math::exp(0.), 1.);
    assert_eq!(Math::exp(-1.), 0.36787944117144233);
    assert_eq!(Math::exp(2.), 7.38905609893065);
}

#[wasm_bindgen_test]
fn expm1() {
    assert_eq!(Math::expm1(0.), 0.);
    assert_eq!(Math::expm1(1.), 1.718281828459045);
    assert_eq!(Math::expm1(-1.), -0.6321205588285577);
    assert_eq!(Math::expm1(2.), 6.38905609893065);
}

#[wasm_bindgen_test]
fn floor() {
    assert_eq!(Math::floor(5.95), 5.);
    assert_eq!(Math::floor(-5.05), -6.);
}

#[wasm_bindgen_test]
fn fround() {
    assert!(Math::fround(5.5) == 5.5);
    assert!(Math::fround(5.05) == 5.050000190734863);
    assert!(Math::fround(5.) == 5.);
    assert!(Math::fround(-5.05) == -5.050000190734863);
}

#[wasm_bindgen_test]
fn hypot() {
    assert!(Math::hypot(3., 4.) == 5.);
    assert!(Math::hypot(3.9, 5.2) == 6.5);
    assert!(Math::hypot(6., 8.) == 10.);
    assert!(Math::hypot(7., 24.) == 25.);
}

#[wasm_bindgen_test]
fn imul() {
    assert!(Math::imul(3, 4) == 12);
    assert!(Math::imul(-5, 12) == -60);
    assert!(Math::imul(0xffffffffu32 as i32, 5) == 0xffffffffu32.wrapping_mul(5) as i32);
}

#[wasm_bindgen_test]
fn log() {
    assert_eq!(Math::log(8.) / Math::log(2.), 3.);
    assert_eq!(Math::log(625.) / Math::log(5.), 4.);
}

#[wasm_bindgen_test]
fn log10() {
    assert_eq!(Math::log10(100000.), 5.);
    assert_eq!(Math::log10(1.), 0.);
    assert_eq!(Math::log10(2.), 0.3010299956639812);
}

#[wasm_bindgen_test]
fn log1p() {
    assert_eq!(Math::log1p(1.), 0.6931471805599453);
    assert_eq!(Math::log1p(0.), 0.);
    assert_eq!(Math::log1p(-1.), NEG_INFINITY);
    assert!(Math::log1p(-2.).is_nan());
}

#[wasm_bindgen_test]
fn log2() {
    assert_eq!(Math::log2(3.), 1.584962500721156);
    assert_eq!(Math::log2(2.), 1.);
    assert_eq!(Math::log2(1.), 0.);
    assert_eq!(Math::log2(0.), NEG_INFINITY);
}

#[wasm_bindgen_test]
fn max() {
    assert_eq!(Math::max(3., 1.), 3.);
    assert_eq!(Math::max(-3., 1.), 1.);
    assert_eq!(Math::max(9913., 43.4), 9913.);
    assert_eq!(Math::max(-27., -43.), -27.);
    assert_eq!(Math::max(-423.27, -43.1), -43.1);
}

#[wasm_bindgen_test]
fn min() {
    assert_eq!(Math::min(3., 1.), 1.);
    assert_eq!(Math::min(-3., 1.), -3.);
    assert_eq!(Math::min(9913., 43.4), 43.4);
    assert_eq!(Math::min(-27., -43.), -43.);
    assert_eq!(Math::min(-423.27, -43.1), -423.27);
}

#[wasm_bindgen_test]
fn pow() {
    assert_eq!(Math::pow(7., 2.), 49.);
    assert_eq!(Math::pow(3.8, 0.5), 3.8f64.powf(0.5f64));
    assert!(Math::pow(-2., 0.5).is_nan());
}

#[wasm_bindgen_test]
fn random() {
    assert!(Math::random() < 1.);
    assert!(Math::random() >= 0.);
}

#[wasm_bindgen_test]
fn round() {
    assert_eq!(Math::round(20.49), 20.);
    assert_eq!(Math::round(20.5), 21.);
    assert_eq!(Math::round(42.), 42.);
    assert_eq!(Math::round(-20.5), -20.);
    assert_eq!(Math::round(-20.51), -21.);
}

#[wasm_bindgen_test]
fn sign() {
    assert_eq!(Math::sign(3.), 1.);
    assert_eq!(Math::sign(-3.), -1.);
    assert_eq!(Math::sign(2.3), 1.);
    assert_eq!(Math::sign(0.), 0.);
    assert!(Math::sign(NAN).is_nan());
}

#[wasm_bindgen_test]
fn sin() {
    assert_eq!(Math::sin(0.), 0.);
    assert_eq!(Math::sin(1.), 1f64.sin());
    assert_eq!(Math::sin(PI / 2.), 1.);
}

#[wasm_bindgen_test]
fn sinh() {
    assert_eq!(Math::sinh(0.), 0.);
    assert_eq!(Math::sinh(1.), 1f64.sinh());
    assert_eq!(Math::sinh(2.3), 2.3f64.sinh());
}

#[wasm_bindgen_test]
fn sqrt() {
    assert_eq!(Math::sqrt(9.), 3.);
    assert_eq!(Math::sqrt(2.), 2f64.sqrt());
    assert_eq!(Math::sqrt(42.42), 42.42f64.sqrt());
    assert_eq!(Math::sqrt(1.), 1.);
    assert!(Math::sqrt(-1.).is_nan());
}

#[wasm_bindgen_test]
fn tan() {
    assert_eq!(Math::tan(0.), 0.);
    assert_eq!(Math::tan(1.), 1f64.tan());
    assert_eq!(Math::tan(0.5), 0.5f64.tan());
}

#[wasm_bindgen_test]
fn tanh() {
    assert_eq!(Math::tanh(0.), 0.);
    assert_eq!(Math::tanh(1.), 1f64.tanh());
    assert_eq!(Math::tanh(0.5), 0.5f64.tanh());
}

#[wasm_bindgen_test]
fn trunc() {
    assert_eq!(Math::trunc(13.37), 13.);
    assert_eq!(Math::trunc(42.84), 42.);
    assert_eq!(Math::trunc(0.123), 0.);
    assert_eq!(Math::trunc(-0.123), 0.);
}
