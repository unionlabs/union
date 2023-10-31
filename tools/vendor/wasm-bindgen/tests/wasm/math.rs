use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/math.js")]
extern "C" {
    fn js_auto_bind_math();

    // There's an identity function called `roundtrip` in the module and we bind
    // that one function with multiple different signatures here. Note that the
    // return value is always `f64` to faithfully capture what was sent to JS
    // (what we're interested in) because all JS numbers fit in `f64` anyway.
    // This is testing what happens when we pass numbers to JS and what it sees.
    #[wasm_bindgen(assert_no_shim, js_name = roundtrip)]
    fn roundtrip_i8(a: i8) -> f64;
    #[wasm_bindgen(assert_no_shim, js_name = roundtrip)]
    fn roundtrip_i16(a: i16) -> f64;
    #[wasm_bindgen(assert_no_shim, js_name = roundtrip)]
    fn roundtrip_i32(a: i32) -> f64;
    #[wasm_bindgen(assert_no_shim, js_name = roundtrip)]
    fn roundtrip_u8(a: u8) -> f64;
    #[wasm_bindgen(assert_no_shim, js_name = roundtrip)]
    fn roundtrip_u16(a: u16) -> f64;
    #[wasm_bindgen(js_name = roundtrip)]
    fn roundtrip_u32(a: u32) -> f64;

    fn test_js_roundtrip();
}

#[wasm_bindgen]
pub fn math(a: f32, b: f64) -> f64 {
    b.acos()
        + b.asin()
        + b.atan()
        + b.atan2(b)
        + b.cbrt()
        + b.cosh()
        + b.exp_m1()
        + b.ln_1p()
        + b.sinh()
        + b.tan()
        + b.tanh()
        + b.hypot(b)
        + b.cos()
        + b.exp()
        + b.exp2()
        + b.mul_add(b, b)
        + b.ln()
        + b.log(b)
        + b.log10()
        + b.log2()
        + b.powi(8)
        + b.powf(b)
        + b.round()
        + b.sin()
        + b.abs()
        + b.signum()
        + b.floor()
        + b.ceil()
        + b.trunc()
        + b.sqrt()
        + (b % (a as f64))
        + ((a.cos()
            + a.exp()
            + a.exp2()
            + a.mul_add(a, a)
            + a.ln()
            + a.log(a)
            + a.log10()
            + a.log2()
            + a.powi(8)
            + a.powf(a)
            + a.round()
            + a.sin()
            + a.abs()
            + a.signum()
            + a.floor()
            + a.ceil()
            + a.trunc()
            + a.sqrt()
            + (a % (b as f32))) as f64)
        + (b + 2.0f64.powf(a as f64))
}

#[wasm_bindgen_test]
fn auto_bind_math() {
    js_auto_bind_math();
}

macro_rules! t_roundtrip {
    ($f:ident($e:expr)) => {
        assert_eq!($f($e), $e as f64)
    };
}

#[wasm_bindgen_test]
fn limits_correct() {
    t_roundtrip!(roundtrip_i8(i8::min_value()));
    t_roundtrip!(roundtrip_i8(0));
    t_roundtrip!(roundtrip_i8(i8::max_value()));
    t_roundtrip!(roundtrip_i16(i16::min_value()));
    t_roundtrip!(roundtrip_i16(0));
    t_roundtrip!(roundtrip_i16(i16::max_value()));
    t_roundtrip!(roundtrip_i32(i32::min_value()));
    t_roundtrip!(roundtrip_i32(0));
    t_roundtrip!(roundtrip_i32(i32::max_value()));
    t_roundtrip!(roundtrip_u8(u8::min_value()));
    t_roundtrip!(roundtrip_u8(0));
    t_roundtrip!(roundtrip_u8(u8::max_value()));
    t_roundtrip!(roundtrip_u16(u16::min_value()));
    t_roundtrip!(roundtrip_u16(0));
    t_roundtrip!(roundtrip_u16(u16::max_value()));
    t_roundtrip!(roundtrip_u32(u32::min_value()));
    t_roundtrip!(roundtrip_u32(0));
    t_roundtrip!(roundtrip_u32(u32::max_value()));

    test_js_roundtrip();

    #[wasm_bindgen]
    pub fn rust_roundtrip_i8(a: i8) -> i8 {
        a
    }
    #[wasm_bindgen]
    pub fn rust_roundtrip_i16(a: i16) -> i16 {
        a
    }
    #[wasm_bindgen]
    pub fn rust_roundtrip_i32(a: i32) -> i32 {
        a
    }
    #[wasm_bindgen]
    pub fn rust_roundtrip_u8(a: u8) -> u8 {
        a
    }
    #[wasm_bindgen]
    pub fn rust_roundtrip_u16(a: u16) -> u16 {
        a
    }
    #[wasm_bindgen]
    pub fn rust_roundtrip_u32(a: u32) -> u32 {
        a
    }
}
