use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = max, js_namespace = Math)]
    static MAX: Function;

    type ArrayPrototype;
    #[wasm_bindgen(method, getter, structural)]
    pub fn push(this: &ArrayPrototype) -> Function;
    #[wasm_bindgen(js_name = prototype, js_namespace = Array)]
    static ARRAY_PROTOTYPE2: ArrayPrototype;
}

#[wasm_bindgen_test]
fn apply() {
    let args = Array::new();
    args.push(&1.into());
    args.push(&2.into());
    args.push(&3.into());
    assert_eq!(MAX.apply(&JsValue::undefined(), &args).unwrap(), 3);

    let arr = JsValue::from(Array::new());
    let args = Array::new();
    args.push(&1.into());
    ARRAY_PROTOTYPE2.push().apply(&arr, &args).unwrap();
    assert_eq!(Array::from(&arr).length(), 1);
}

#[wasm_bindgen(module = "tests/wasm/Function.js")]
extern "C" {
    fn get_function_to_bind() -> Function;
    fn get_value_to_bind_to() -> JsValue;
    fn list() -> Function;
    fn add_arguments() -> Function;
    fn call_function(f: &Function) -> JsValue;
    fn call_function_arg(f: &Function, arg0: JsValue) -> JsValue;

}

#[wasm_bindgen_test]
fn bind() {
    let f = get_function_to_bind();
    let new_f = f.bind(&get_value_to_bind_to());
    assert_eq!(call_function(&f), 1);
    assert_eq!(call_function(&new_f), 2);
}

#[wasm_bindgen_test]
fn bind0() {
    let f = get_function_to_bind();
    let new_f = f.bind0(&get_value_to_bind_to());
    assert_eq!(call_function(&f), 1);
    assert_eq!(call_function(&new_f), 2);
}

#[wasm_bindgen_test]
fn bind1() {
    let a_list = list();
    let prepended_list = a_list.bind1(&JsValue::NULL, &JsValue::from(2));

    assert_eq!(Array::from(&call_function(&prepended_list)).pop(), 2);

    let adder = add_arguments();
    let add_42 = adder.bind1(&JsValue::NULL, &JsValue::from(42));

    assert_eq!(call_function_arg(&add_42, JsValue::from(1)), 43);
    assert_eq!(call_function_arg(&add_42, JsValue::from(378)), 420);
}

#[wasm_bindgen_test]
fn bind2() {
    let a_list = list();
    let prepended_list = a_list.bind2(&JsValue::NULL, &JsValue::from(2), &JsValue::from(3));

    let arr = Array::from(&call_function(&prepended_list));

    assert_eq!(arr.pop(), 3);
    assert_eq!(arr.pop(), 2);

    let adder = add_arguments();
    let always_69 = adder.bind2(&JsValue::NULL, &JsValue::from(66), &JsValue::from(3));

    assert_eq!(call_function(&always_69), 69);
}

#[wasm_bindgen_test]
fn bind3() {
    let a_list = list();
    let prepended_list = a_list.bind3(
        &JsValue::NULL,
        &JsValue::from(2),
        &JsValue::from(3),
        &JsValue::from(4),
    );

    let arr = Array::from(&call_function(&prepended_list));

    assert_eq!(arr.pop(), 4);
    assert_eq!(arr.pop(), 3);
    assert_eq!(arr.pop(), 2);

    let adder = add_arguments();
    let always_69 = adder.bind2(&JsValue::NULL, &JsValue::from(66), &JsValue::from(3));

    assert_eq!(call_function(&always_69), 69);
}

#[wasm_bindgen_test]
fn length() {
    assert_eq!(MAX.length(), 2);
    assert_eq!(ARRAY_PROTOTYPE2.push().length(), 1);
}

#[wasm_bindgen_test]
fn name() {
    assert_eq!(JsValue::from(MAX.name()), "max");
    assert_eq!(JsValue::from(ARRAY_PROTOTYPE2.push().name()), "push");
}

#[wasm_bindgen_test]
fn to_string() {
    assert!(MAX.to_string().length() > 0);
}

#[wasm_bindgen_test]
fn function_inheritance() {
    assert!(MAX.is_instance_of::<Function>());
    assert!(MAX.is_instance_of::<Object>());
    let _: &Object = MAX.as_ref();
}
