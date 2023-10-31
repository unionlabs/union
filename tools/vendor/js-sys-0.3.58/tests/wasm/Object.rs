use js_sys::*;
use std::f64::NAN;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

#[wasm_bindgen]
extern "C" {
    type Foo42;
    #[wasm_bindgen(method, setter, structural)]
    fn set_foo(this: &Foo42, val: JsValue);

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

#[wasm_bindgen(module = "tests/wasm/Object.js")]
extern "C" {
    fn map_with_symbol_key() -> Object;
    fn symbol_key() -> JsValue;

    type Foo;
    #[wasm_bindgen(constructor)]
    fn new() -> Foo;

    #[wasm_bindgen(js_name = prototype, js_namespace = Foo)]
    static FOO_PROTOTYPE: Object;
    #[wasm_bindgen(js_name = prototype, js_namespace = Bar)]
    static BAR_PROTOTYPE: Object;
}

fn foo_42() -> Object {
    let foo = Foo42::from(JsValue::from(Object::new()));
    foo.set_foo(42.into());
    JsValue::from(foo).into()
}

#[wasm_bindgen_test]
fn new() {
    assert!(JsValue::from(Object::new()).is_object());
}

#[wasm_bindgen_test]
fn assign() {
    let a = JsValue::from("a");
    let b = JsValue::from("b");
    let c = JsValue::from("c");

    let target = Object::new();
    Reflect::set(target.as_ref(), a.as_ref(), a.as_ref()).unwrap();

    let src1 = Object::new();
    Reflect::set(src1.as_ref(), &a, &c).unwrap();

    let src2 = Object::new();
    Reflect::set(src2.as_ref(), &b, &b).unwrap();

    let src3 = Object::new();
    Reflect::set(src3.as_ref(), &c, &c).unwrap();

    let res = Object::assign3(&target, &src1, &src2, &src3);

    assert!(Object::is(target.as_ref(), res.as_ref()));
    assert_eq!(Reflect::get(target.as_ref(), &a).unwrap(), c);
    assert_eq!(Reflect::get(target.as_ref(), &b).unwrap(), b);
    assert_eq!(Reflect::get(target.as_ref(), &c).unwrap(), c);
}

#[wasm_bindgen_test]
fn create() {
    let array_proto = eval("Array.prototype")
        .unwrap()
        .dyn_into::<Object>()
        .unwrap();
    let my_array = Object::create(&array_proto);
    assert!(my_array.is_instance_of::<Array>());
}

#[wasm_bindgen_test]
fn define_property() {
    let value = DefinePropertyAttrs::from(JsValue::from(Object::new()));
    value.set_value(&43.into());
    let descriptor = Object::from(JsValue::from(value));
    let foo = foo_42();
    let foo = Object::define_property(&foo, &"bar".into(), &descriptor);
    assert!(foo.has_own_property(&"bar".into()));
}

#[wasm_bindgen_test]
fn define_properties() {
    let props = Object::new();
    let descriptor = DefinePropertyAttrs::from(JsValue::from(Object::new()));
    descriptor.set_value(&42.into());
    let descriptor = JsValue::from(descriptor);
    Reflect::set(props.as_ref(), &JsValue::from("bar"), &descriptor).unwrap();
    Reflect::set(props.as_ref(), &JsValue::from("car"), &descriptor).unwrap();
    let foo = foo_42();
    let foo = Object::define_properties(&foo, &props);
    assert!(foo.has_own_property(&"bar".into()));
    assert!(foo.has_own_property(&"car".into()));
}

#[wasm_bindgen_test]
fn entries() {
    let entries = Object::entries(&foo_42());
    assert_eq!(entries.length(), 1);
    entries.for_each(&mut |x, _, _| {
        assert!(x.is_object());
        let array: Array = x.into();
        assert_eq!(array.shift(), "foo");
        assert_eq!(array.shift(), 42);
        assert_eq!(array.length(), 0);
    });
}

#[wasm_bindgen_test]
fn from_entries() {
    let array = Array::new();
    let entry_one = Array::new();
    let entry_two = Array::new();
    entry_one.push(&"foo".into());
    entry_one.push(&"bar".into());
    entry_two.push(&"baz".into());
    entry_two.push(&42.into());
    array.push(&entry_one);
    array.push(&entry_two);
    let object = Object::from_entries(&array).unwrap();

    assert_eq!(Reflect::get(object.as_ref(), &"foo".into()).unwrap(), "bar");
    assert_eq!(Reflect::get(object.as_ref(), &"baz".into()).unwrap(), 42);

    let not_iterable = Object::new();
    let error = Object::from_entries(&not_iterable).unwrap_err();
    assert!(error.is_instance_of::<TypeError>());
}

#[wasm_bindgen_test]
fn get_own_property_descriptor() {
    let foo = foo_42();
    let desc = Object::get_own_property_descriptor(&foo, &"foo".into());
    assert_eq!(PropertyDescriptor::from(desc).value(), 42);
    let desc = Object::get_own_property_descriptor(&foo, &"bar".into());
    assert!(desc.is_undefined());
}

#[wasm_bindgen_test]
fn get_own_property_descriptors() {
    let foo = foo_42();
    let descriptors = Object::get_own_property_descriptors(&foo);
    let foo_desc = Reflect::get(&descriptors, &"foo".into()).unwrap();
    assert_eq!(PropertyDescriptor::from(foo_desc).value(), 42);
}

#[wasm_bindgen_test]
fn get_own_property_names() {
    let names = Object::get_own_property_names(&foo_42());
    assert_eq!(names.length(), 1);
    names.for_each(&mut |x, _, _| {
        assert_eq!(x, "foo");
    });
}

#[wasm_bindgen_test]
fn get_own_property_symbols() {
    let symbols = Object::get_own_property_symbols(&map_with_symbol_key());
    assert_eq!(symbols.length(), 1);
}

#[wasm_bindgen_test]
fn get_prototype_of() {
    let proto = JsValue::from(Object::get_prototype_of(&Object::new().into()));
    assert_eq!(proto, *OBJECT_PROTOTYPE);
    let proto = JsValue::from(Object::get_prototype_of(&Array::new().into()));
    assert_eq!(proto, *ARRAY_PROTOTYPE);
}

#[wasm_bindgen_test]
fn has_own_property() {
    assert!(foo_42().has_own_property(&"foo".into()));
    assert!(!foo_42().has_own_property(&"bar".into()));
    assert!(map_with_symbol_key().has_own_property(&symbol_key()));
}

#[wasm_bindgen_test]
fn has_own() {
    assert!(Object::has_own(&foo_42(), &"foo".into()));
    assert!(!Object::has_own(&foo_42(), &"bar".into()));
    assert!(Object::has_own(&map_with_symbol_key(), &symbol_key()));
}

#[wasm_bindgen_test]
fn to_string() {
    assert_eq!(Object::new().to_string(), "[object Object]");
    assert_eq!(foo_42().to_string(), "[object Object]");
}

#[wasm_bindgen_test]
fn is() {
    let object = JsValue::from(Object::new());
    assert!(Object::is(&object, &object));
    assert!(Object::is(&JsValue::undefined(), &JsValue::undefined()));
    assert!(Object::is(&JsValue::null(), &JsValue::null()));
    assert!(Object::is(&JsValue::TRUE, &JsValue::TRUE));
    assert!(Object::is(&JsValue::FALSE, &JsValue::FALSE));
    assert!(Object::is(&"foo".into(), &"foo".into()));
    assert!(Object::is(&JsValue::from(42), &JsValue::from(42)));
    assert!(Object::is(&JsValue::from(NAN), &JsValue::from(NAN)));

    let another_object = JsValue::from(Object::new());
    assert!(!Object::is(&object, &another_object));
    assert!(!Object::is(&JsValue::TRUE, &JsValue::FALSE));
    assert!(!Object::is(&"foo".into(), &"bar".into()));
    assert!(!Object::is(&JsValue::from(23), &JsValue::from(42)));
}

#[wasm_bindgen_test]
fn is_extensible() {
    let object = Object::new();
    assert!(Object::is_extensible(&object));
    Object::prevent_extensions(&object);
    assert!(!Object::is_extensible(&object));
}

#[wasm_bindgen_test]
fn is_frozen() {
    let object = Object::new();
    assert!(!Object::is_frozen(&object));
    Object::freeze(&object);
    assert!(Object::is_frozen(&object));
}

#[wasm_bindgen_test]
fn is_sealed() {
    let object = Object::new();
    assert!(!Object::is_sealed(&object));
    Object::seal(&object);
    assert!(Object::is_sealed(&object));
}

#[wasm_bindgen_test]
fn is_prototype_of() {
    let foo = JsValue::from(Foo::new());
    assert!(FOO_PROTOTYPE.is_prototype_of(&foo));
    assert!(!BAR_PROTOTYPE.is_prototype_of(&foo));
}

#[wasm_bindgen_test]
fn keys() {
    let keys = Object::keys(&foo_42());
    assert_eq!(keys.length(), 1);
    keys.for_each(&mut |x, _, _| {
        assert_eq!(x, "foo");
    });
}

#[wasm_bindgen_test]
fn values() {
    let values = Object::values(&foo_42());
    assert_eq!(values.length(), 1);
    values.for_each(&mut |x, _, _| {
        assert_eq!(x, 42);
    });
}

#[wasm_bindgen_test]
fn property_is_enumerable() {
    assert!(foo_42().property_is_enumerable(&"foo".into()));
    assert!(!foo_42().property_is_enumerable(&42.into()));
    assert!(!Object::new().property_is_enumerable(&"foo".into()));
}

#[wasm_bindgen_test]
fn set_prototype_of() {
    let a = foo_42();
    let b = foo_42();
    Object::set_prototype_of(&a, &b);
    assert!(b.is_prototype_of(&a.into()));
}

#[wasm_bindgen_test]
fn to_locale_string() {
    assert_eq!(Object::new().to_locale_string(), "[object Object]");
}

#[wasm_bindgen_test]
fn value_of() {
    let a = JsValue::from(foo_42());
    let b = JsValue::from(foo_42());
    let a2 = JsValue::from(Object::from(a.clone()).value_of());
    assert_eq!(a, a);
    assert_eq!(a, a2);
    assert_ne!(a, b);
    assert_ne!(a2, b);
}
