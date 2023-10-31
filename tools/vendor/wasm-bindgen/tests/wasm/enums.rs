use self::inner::ColorWithCustomValues;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/enums.js")]
extern "C" {
    fn js_c_style_enum();
    fn js_c_style_enum_with_custom_values();
    fn js_handle_optional_enums(x: Option<Color>) -> Option<Color>;
    fn js_expect_enum(x: Color, y: Option<Color>);
    fn js_expect_enum_none(x: Option<Color>);
    fn js_renamed_enum(b: RenamedEnum);
}

#[wasm_bindgen]
#[derive(PartialEq, Debug)]
pub enum Color {
    Green,
    Yellow,
    Red,
}

pub mod inner {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub enum ColorWithCustomValues {
        Green = 21,
        Yellow = 34,
        Red = 2,
    }
}

#[wasm_bindgen(js_name = JsRenamedEnum)]
#[derive(Copy, Clone)]
pub enum RenamedEnum {
    A = 10,
    B = 20,
}

#[wasm_bindgen]
pub fn enum_cycle(color: Color) -> Color {
    match color {
        Color::Green => Color::Yellow,
        Color::Yellow => Color::Red,
        Color::Red => Color::Green,
    }
}

#[wasm_bindgen]
pub fn enum_with_custom_values_cycle(color: ColorWithCustomValues) -> ColorWithCustomValues {
    match color {
        ColorWithCustomValues::Green => ColorWithCustomValues::Yellow,
        ColorWithCustomValues::Yellow => ColorWithCustomValues::Red,
        ColorWithCustomValues::Red => ColorWithCustomValues::Green,
    }
}

#[wasm_bindgen_test]
fn c_style_enum() {
    js_c_style_enum();
}

#[wasm_bindgen_test]
fn c_style_enum_with_custom_values() {
    js_c_style_enum_with_custom_values();
}

#[wasm_bindgen]
pub fn handle_optional_enums(x: Option<Color>) -> Option<Color> {
    x
}

#[wasm_bindgen_test]
fn test_optional_enums() {
    use self::Color::*;

    assert_eq!(js_handle_optional_enums(None), None);
    assert_eq!(js_handle_optional_enums(Some(Green)), Some(Green));
    assert_eq!(js_handle_optional_enums(Some(Yellow)), Some(Yellow));
    assert_eq!(js_handle_optional_enums(Some(Red)), Some(Red));
}

#[wasm_bindgen_test]
fn test_optional_enum_values() {
    use self::Color::*;

    js_expect_enum(Green, Some(Green));
    js_expect_enum(Yellow, Some(Yellow));
    js_expect_enum(Red, Some(Red));
    js_expect_enum_none(None);
}

#[wasm_bindgen_test]
fn test_renamed_enum() {
    js_renamed_enum(RenamedEnum::B);
}
