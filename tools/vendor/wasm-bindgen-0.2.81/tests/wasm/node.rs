use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/node.js")]
extern "C" {
    fn test_works();
    static FOO: JsValue;
    fn hit();
}

#[wasm_bindgen_test]
fn works() {
    hit();
    assert_eq!(FOO.as_f64(), Some(1.0));
    test_works();
}

#[wasm_bindgen]
pub struct Foo {
    contents: u32,
}

#[wasm_bindgen]
impl Foo {
    pub fn new() -> Foo {
        Foo::with_contents(0)
    }
    pub fn with_contents(a: u32) -> Foo {
        Foo { contents: a }
    }
    pub fn add(&mut self, amt: u32) -> u32 {
        self.contents += amt;
        self.contents
    }
}

#[wasm_bindgen]
pub enum Color {
    Green,
    Yellow,
    Red,
}
#[wasm_bindgen]
pub fn cycle(color: Color) -> Color {
    match color {
        Color::Green => Color::Yellow,
        Color::Yellow => Color::Red,
        Color::Red => Color::Green,
    }
}

#[wasm_bindgen]
pub fn node_math(a: f32, b: f64) -> f64 {
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
