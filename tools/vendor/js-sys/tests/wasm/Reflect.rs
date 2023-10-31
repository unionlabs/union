use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/Reflect.js")]
extern "C" {
    fn get_char_at() -> Function;

    #[wasm_bindgen(js_name = Rectangle)]
    static RECTANGLE_CLASS: Function;
    #[wasm_bindgen(js_name = Rectangle2)]
    static RECTANGLE2_CLASS: Function;

    #[derive(Clone)]
    type Rectangle;
    #[wasm_bindgen(constructor)]
    fn new() -> Rectangle;
    #[wasm_bindgen(method, getter, structural)]
    fn x(this: &Rectangle) -> u32;
    #[wasm_bindgen(method, getter, structural, js_name = x)]
    fn x_jsval(this: &Rectangle) -> JsValue;
    #[wasm_bindgen(method, setter, structural)]
    fn set_x(this: &Rectangle, x: u32);

    fn throw_all_the_time() -> Object;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = prototype, js_namespace = Object)]
    static OBJECT_PROTOTYPE: JsValue;
    #[wasm_bindgen(js_name = prototype, js_namespace = Array)]
    static ARRAY_PROTOTYPE: JsValue;

    type DefinePropertyAttrs;
    #[wasm_bindgen(method, setter, structural)]
    fn set_value(this: &DefinePropertyAttrs, val: &JsValue);

    type PropertyDescriptor;
    #[wasm_bindgen(method, getter, structural)]
    fn value(this: &PropertyDescriptor) -> JsValue;
}

#[wasm_bindgen_test]
fn apply() {
    let args = Array::new();
    args.push(&3.into());
    assert_eq!(
        Reflect::apply(&get_char_at(), &"ponies".into(), &args).unwrap(),
        "i"
    );
}

#[wasm_bindgen_test]
fn construct() {
    let args = Array::new();
    args.push(&10.into());
    args.push(&10.into());
    let instance = Reflect::construct(&RECTANGLE_CLASS, &args).unwrap();
    assert_eq!(Rectangle::from(instance).x(), 10);
}

#[wasm_bindgen_test]
fn construct_with_new_target() {
    let args = Array::new();
    args.push(&10.into());
    args.push(&10.into());
    let instance =
        Reflect::construct_with_new_target(&RECTANGLE_CLASS, &args, &RECTANGLE2_CLASS).unwrap();
    assert_eq!(Rectangle::from(instance).x(), 10);
}

#[wasm_bindgen_test]
fn define_property() {
    let value = DefinePropertyAttrs::from(JsValue::from(Object::new()));
    value.set_value(&42.into());
    let obj = Object::from(JsValue::from(value));
    assert!(Reflect::define_property(&obj, &"key".into(), &obj).unwrap());
}

#[wasm_bindgen_test]
fn delete_property() {
    let r = Rectangle::new();
    r.set_x(10);

    let obj = Object::from(JsValue::from(r.clone()));
    Reflect::delete_property(&obj, &"x".into()).unwrap();
    assert!(r.x_jsval().is_undefined());

    let array = Array::new();
    array.push(&1.into());
    let obj = Object::from(JsValue::from(array));
    Reflect::delete_property(&obj, &0.into()).unwrap();
    let array = Array::from(&JsValue::from(obj));
    assert!(array.length() == 1);
    array.for_each(&mut |x, _, _| assert!(x.is_undefined()));
}

#[wasm_bindgen_test]
fn get() {
    let r = Rectangle::new();
    r.set_x(10);

    let obj = JsValue::from(r.clone());
    assert_eq!(Reflect::get(&obj, &"x".into()).unwrap(), 10);
}

#[wasm_bindgen_test]
fn get_f64() {
    let a = Array::new();
    assert_eq!(Reflect::get_f64(&a, 0.0).unwrap(), JsValue::UNDEFINED);
    assert_eq!(a.push(&JsValue::from_str("Hi!")), 1);
    assert_eq!(Reflect::get_f64(&a, 0.0).unwrap(), JsValue::from_str("Hi!"));
}

#[wasm_bindgen_test]
fn get_u32() {
    let a = Array::new();
    assert_eq!(Reflect::get_u32(&a, 0).unwrap(), JsValue::UNDEFINED);
    assert_eq!(a.push(&JsValue::from_str("Hi!")), 1);
    assert_eq!(Reflect::get_u32(&a, 0).unwrap(), JsValue::from_str("Hi!"));
}

#[wasm_bindgen_test]
fn get_own_property_descriptor() {
    let r = Rectangle::new();
    r.set_x(10);

    let obj = Object::from(JsValue::from(r.clone()));
    let desc = Reflect::get_own_property_descriptor(&obj, &"x".into()).unwrap();
    assert_eq!(PropertyDescriptor::from(desc).value(), 10);
    let desc = Reflect::get_own_property_descriptor(&obj, &"foo".into()).unwrap();
    assert!(desc.is_undefined());
}

#[wasm_bindgen_test]
fn get_prototype_of() {
    let proto = JsValue::from(Reflect::get_prototype_of(&Object::new().into()).unwrap());
    assert_eq!(proto, *OBJECT_PROTOTYPE);
    let proto = JsValue::from(Reflect::get_prototype_of(&Array::new().into()).unwrap());
    assert_eq!(proto, *ARRAY_PROTOTYPE);
}

#[wasm_bindgen_test]
fn has() {
    let obj = JsValue::from(Rectangle::new());
    assert!(Reflect::has(&obj, &"x".into()).unwrap());
    assert!(!Reflect::has(&obj, &"foo".into()).unwrap());
}

#[wasm_bindgen_test]
fn is_extensible() {
    let obj = Object::from(JsValue::from(Rectangle::new()));
    assert!(Reflect::is_extensible(&obj).unwrap());
    Reflect::prevent_extensions(&obj).unwrap();
    assert!(!Reflect::is_extensible(&obj).unwrap());
    let obj = Object::seal(&Object::new());
    assert!(!Reflect::is_extensible(&obj).unwrap());
}

#[wasm_bindgen_test]
fn own_keys() {
    let obj = JsValue::from(Rectangle::new());
    let keys = Reflect::own_keys(&obj).unwrap();
    assert!(keys.length() == 2);
    keys.for_each(&mut |k, _, _| {
        assert!(k == "x" || k == "y");
    });
}

#[wasm_bindgen_test]
fn prevent_extensions() {
    let obj = Object::new();
    Reflect::prevent_extensions(&obj).unwrap();
    assert!(!Reflect::is_extensible(&obj).unwrap());
}

#[wasm_bindgen_test]
fn set() {
    let obj = JsValue::from(Object::new());
    assert!(Reflect::set(&obj, &"key".into(), &"value".into()).unwrap());
    assert_eq!(Reflect::get(&obj, &"key".into()).unwrap(), "value");
}

#[wasm_bindgen_test]
fn set_f64() {
    let a = Array::new();
    a.push(&JsValue::from_str("Hi!"));

    assert_eq!(Reflect::get_f64(&a, 0.0).unwrap(), JsValue::from_str("Hi!"));

    Reflect::set_f64(&a, 0.0, &JsValue::from_str("Bye!")).unwrap();

    assert_eq!(
        Reflect::get_f64(&a, 0.0).unwrap(),
        JsValue::from_str("Bye!")
    );
}

#[wasm_bindgen_test]
fn set_u32() {
    let a = Array::new();
    a.push(&JsValue::from_str("Hi!"));

    assert_eq!(Reflect::get_u32(&a, 0).unwrap(), JsValue::from_str("Hi!"));

    Reflect::set_u32(&a, 0, &JsValue::from_str("Bye!")).unwrap();

    assert_eq!(Reflect::get_u32(&a, 0).unwrap(), JsValue::from_str("Bye!"));
}

#[wasm_bindgen_test]
fn set_with_receiver() {
    let obj1 = JsValue::from(Object::new());
    let obj2 = JsValue::from(Object::new());
    assert!(Reflect::set_with_receiver(&obj2, &"key".into(), &"value".into(), &obj2).unwrap());
    assert!(Reflect::get(&obj1, &"key".into()).unwrap().is_undefined());
    assert_eq!(Reflect::get(&obj2, &"key".into()).unwrap(), "value");
}

#[wasm_bindgen_test]
fn set_prototype_of() {
    let obj = Object::new();
    assert!(Reflect::set_prototype_of(&obj, &JsValue::null()).unwrap());
    let obj = JsValue::from(obj);
    assert_eq!(
        JsValue::from(Reflect::get_prototype_of(&obj).unwrap()),
        JsValue::null()
    );
}

#[wasm_bindgen_test]
fn reflect_bindings_handle_proxies_that_just_throw_for_everything() {
    let p = throw_all_the_time();

    let desc = Object::new();
    Reflect::set(desc.as_ref(), &"value".into(), &1.into()).unwrap();
    assert!(Reflect::define_property(&p, &"a".into(), &desc).is_err());

    assert!(Reflect::delete_property(&p, &"a".into()).is_err());

    assert!(Reflect::get(p.as_ref(), &"a".into()).is_err());
    assert!(Reflect::get_f64(p.as_ref(), 0.0).is_err());
    assert!(Reflect::get_u32(p.as_ref(), 0).is_err());

    assert!(Reflect::get_own_property_descriptor(&p, &"a".into()).is_err());

    assert!(Reflect::get_prototype_of(p.as_ref()).is_err());

    assert!(Reflect::has(p.as_ref(), &"a".into()).is_err());

    assert!(Reflect::is_extensible(&p).is_err());

    assert!(Reflect::own_keys(p.as_ref()).is_err());

    assert!(Reflect::prevent_extensions(&p).is_err());

    assert!(Reflect::set(p.as_ref(), &"a".into(), &1.into()).is_err());
    assert!(Reflect::set_f64(p.as_ref(), 0.0, &1.into()).is_err());
    assert!(Reflect::set_u32(p.as_ref(), 0, &1.into()).is_err());

    assert!(Reflect::set_prototype_of(&p, Object::new().as_ref()).is_err());
}
