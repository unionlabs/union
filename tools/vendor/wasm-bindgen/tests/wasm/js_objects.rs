use js_sys::JsString;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/js_objects.js")]
extern "C" {
    fn simple_foo(s: &JsValue);
    fn js_simple();

    fn owned_foo(s: JsValue);
    fn js_owned();

    fn clone_foo1(s: JsValue);
    fn clone_foo2(s: &JsValue);
    fn clone_foo3(s: JsValue);
    fn clone_foo4(s: &JsValue);
    fn clone_foo5(s: JsValue);
    fn js_clone();

    fn promote_foo1(s: &JsValue);
    fn promote_foo2(s: JsValue);
    fn promote_foo3(s: &JsValue);
    fn promote_foo4(s: JsValue);
    fn js_promote();

    fn returning_vector_foo() -> JsValue;
    fn js_returning_vector();
    fn js_another_vector_return();

    fn returning_vector_string_foo() -> JsString;
    fn js_returning_vector_string();
    fn js_another_vector_string_return();

    fn verify_serde(val: JsValue) -> JsValue;
}

#[wasm_bindgen]
pub fn simple_bar(s: &JsValue) {
    simple_foo(s);
}

#[wasm_bindgen_test]
fn simple() {
    js_simple();
}

#[wasm_bindgen]
pub fn owned_bar(s: JsValue) {
    owned_foo(s);
}

#[wasm_bindgen_test]
fn owned() {
    js_owned();
}

#[wasm_bindgen]
pub fn clone_bar(s: JsValue) {
    clone_foo1(s.clone());
    clone_foo2(&s);
    clone_foo3(s.clone());
    clone_foo4(&s);
    clone_foo5(s);
}

#[wasm_bindgen_test]
fn clone() {
    js_clone();
}

#[wasm_bindgen]
pub fn promote_bar(s: &JsValue) {
    promote_foo1(s);
    promote_foo2(s.clone());
    promote_foo3(s);
    promote_foo4(s.clone());
}

#[wasm_bindgen_test]
fn promote() {
    js_promote();
}

#[wasm_bindgen]
pub fn returning_vector_bar() -> Vec<JsValue> {
    let mut res = Vec::new();
    for _ in 0..10 {
        res.push(returning_vector_foo())
    }
    res
}

#[wasm_bindgen_test]
fn returning_vector() {
    js_returning_vector();
}

#[wasm_bindgen]
pub fn another_vector_return_get_array() -> Vec<JsValue> {
    vec![
        JsValue::from(1),
        JsValue::from(2),
        JsValue::from(3),
        JsValue::from(4),
        JsValue::from(5),
        JsValue::from(6),
    ]
}

#[wasm_bindgen_test]
fn another_vector_return() {
    js_another_vector_return();
}

#[wasm_bindgen]
pub fn returning_vector_string_bar() -> Vec<JsString> {
    let mut res = Vec::new();
    for _ in 0..10 {
        res.push(returning_vector_string_foo())
    }
    res
}

#[wasm_bindgen_test]
fn returning_vector_string() {
    js_returning_vector_string();
}

#[wasm_bindgen]
pub fn another_vector_string_return_get_array() -> Vec<JsString> {
    vec![
        "1".into(),
        "2".into(),
        "3".into(),
        "4".into(),
        "5".into(),
        "6".into(),
    ]
}

#[wasm_bindgen_test]
fn another_vector_string_return() {
    js_another_vector_string_return();
}

#[cfg(feature = "serde-serialize")]
#[wasm_bindgen_test]
#[allow(deprecated)]
fn serde() {
    #[derive(Deserialize, Serialize)]
    pub struct SerdeFoo {
        a: u32,
        b: String,
        c: Option<SerdeBar>,
        d: SerdeBar,
    }

    #[derive(Deserialize, Serialize)]
    pub struct SerdeBar {
        a: u32,
    }

    let js = JsValue::from_serde("foo").unwrap();
    assert_eq!(js.as_string(), Some("foo".to_string()));

    let ret = verify_serde(
        JsValue::from_serde(&SerdeFoo {
            a: 0,
            b: "foo".to_string(),
            c: None,
            d: SerdeBar { a: 1 },
        })
        .unwrap(),
    );

    let result = ret.into_serde::<SerdeFoo>().unwrap();
    assert_eq!(result.a, 2);
    assert_eq!(result.b, "bar");
    assert!(result.c.is_some());
    assert_eq!(result.c.as_ref().unwrap().a, 3);
    assert_eq!(result.d.a, 4);

    assert_eq!(JsValue::from("bar").into_serde::<String>().unwrap(), "bar");
    assert_eq!(JsValue::undefined().into_serde::<i32>().ok(), None);
}
