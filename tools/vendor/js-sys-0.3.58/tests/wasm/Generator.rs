use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/Generator.js")]
extern "C" {
    fn one_two_generator() -> Generator;
    fn dummy_generator() -> Generator;
    fn broken_generator() -> Generator;

    type GeneratorResult;

    #[wasm_bindgen(method, getter, structural)]
    fn value(this: &GeneratorResult) -> JsValue;
    #[wasm_bindgen(method, getter, structural)]
    fn done(this: &GeneratorResult) -> bool;

}

#[wasm_bindgen_test]
fn return_() {
    let gen = one_two_generator();
    gen.next(&JsValue::undefined()).unwrap();

    let res = GeneratorResult::from(gen.return_(&42.into()));
    assert_eq!(res.value(), 42);
    assert!(res.done());

    let next = GeneratorResult::from(gen.next(&JsValue::undefined()).unwrap());
    assert!(next.value().is_undefined());
    assert!(next.done());
}

#[wasm_bindgen_test]
fn next() {
    let gen = dummy_generator();

    let result = GeneratorResult::from(gen.next(&JsValue::undefined()).unwrap());
    assert!(!result.done());
    assert_eq!(result.value(), "2 * 2");

    let result = GeneratorResult::from(gen.next(&4.into()).unwrap());
    assert!(result.done());
    assert_eq!(result.value(), true);

    assert!(broken_generator().next(&3.into()).is_err());
}

#[wasm_bindgen_test]
fn throw() {
    let gen = one_two_generator();
    gen.next(&JsValue::undefined()).unwrap();

    assert!(gen.throw(&Error::new("something went wrong")).is_err());
    let next = GeneratorResult::from(gen.next(&JsValue::undefined()).unwrap());
    assert!(next.value().is_undefined());
    assert!(next.done());
}

#[wasm_bindgen_test]
fn generator_inheritance() {
    let gen = dummy_generator();

    assert!(gen.is_instance_of::<Object>());
}
