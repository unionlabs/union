//! dox

#![deny(missing_docs)] // test that documenting public bindings is enough
#![allow(clippy::redundant_clone)] // test specifically with cloned objects

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/import_class.js")]
extern "C" {
    fn math_log(f: f64) -> f64;

    #[wasm_bindgen(js_namespace = StaticFunction)]
    fn bar() -> u32;

    #[derive(Clone)]
    type Construct;
    #[wasm_bindgen(js_namespace = Construct)]
    fn create() -> Construct;
    #[wasm_bindgen(method)]
    fn get_internal_string(this: &Construct) -> String;
    #[wasm_bindgen(method)]
    fn append_to_internal_string(this: &Construct, s: &str);
    #[wasm_bindgen(method)]
    fn assert_internal_string(this: &Construct, s: &str);

    #[wasm_bindgen(method, js_name = "kebab-case")]
    fn kebab_case(this: &Construct) -> u32;
    #[wasm_bindgen(method, getter, js_name = "kebab-case-val")]
    fn kebab_case_val(this: &Construct) -> u32;
    #[wasm_bindgen(method, setter, js_name = "kebab-case-val")]
    fn set_kebab_case_val(this: &Construct, val: u32);
    #[wasm_bindgen(static_method_of = Construct, js_name = "static-kebab-case")]
    fn static_kebab_case() -> u32;
    #[wasm_bindgen(static_method_of = Construct, getter, js_name = "static-kebab-case-val")]
    fn static_kebab_case_val() -> u32;
    #[wasm_bindgen(static_method_of = Construct, setter, js_name = "static-kebab-case-val")]
    fn set_static_kebab_case_val(val: u32);

    type NewConstructors;
    #[wasm_bindgen(constructor)]
    fn new(arg: i32) -> NewConstructors;
    #[wasm_bindgen(method)]
    fn get(this: &NewConstructors) -> i32;

    #[wasm_bindgen(js_name = default)]
    type RenamedTypes;
    #[wasm_bindgen(constructor, js_class = default)]
    fn new(arg: i32) -> RenamedTypes;
    #[wasm_bindgen(method, js_class = default)]
    fn get(this: &RenamedTypes) -> i32;

    fn switch_methods_a();
    fn switch_methods_b();
    type SwitchMethods;
    #[wasm_bindgen(constructor)]
    #[wasm_bindgen(final)]
    fn new() -> SwitchMethods;
    #[wasm_bindgen(js_namespace = SwitchMethods, final)]
    fn a();
    fn switch_methods_called() -> bool;
    #[wasm_bindgen(method, final)]
    fn b(this: &SwitchMethods);

    type Properties;
    #[wasm_bindgen(constructor)]
    fn new() -> Properties;
    #[wasm_bindgen(getter, method)]
    fn a(this: &Properties) -> i32;
    #[wasm_bindgen(setter, method)]
    fn set_a(this: &Properties, a: i32);

    type RenameProperties;
    #[wasm_bindgen(constructor)]
    fn new() -> RenameProperties;
    #[wasm_bindgen(getter = a, method)]
    fn test(this: &RenameProperties) -> i32;
    #[wasm_bindgen(getter, method, js_name = a)]
    fn test2(this: &RenameProperties) -> i32;
    #[wasm_bindgen(setter = a, method)]
    fn another(this: &RenameProperties, a: i32);
    #[wasm_bindgen(setter, method, js_name = a)]
    fn another2(this: &RenameProperties, a: i32);

    /// dox
    pub type AssertImportDenyDocsWorks;
    /// dox
    #[wasm_bindgen(constructor)]
    pub fn new() -> AssertImportDenyDocsWorks;
    /// dox
    #[wasm_bindgen(getter = a, method)]
    pub fn test(this: &AssertImportDenyDocsWorks) -> i32;
    /// dox
    pub fn foo();

    pub type Options;
    #[wasm_bindgen(constructor)]
    fn new() -> Options;
    fn take_none(val: Option<Options>);
    fn take_some(val: Option<Options>);
    fn return_null() -> Option<Options>;
    fn return_undefined() -> Option<Options>;
    fn return_some() -> Option<Options>;
    fn run_rust_option_tests();

    type CatchConstructors;
    #[wasm_bindgen(constructor, catch)]
    fn new(x: u32) -> Result<CatchConstructors, JsValue>;

    type StaticStructural;
    #[wasm_bindgen(static_method_of = StaticStructural, structural)]
    fn static_structural(a: u32) -> u32;

    #[derive(Clone)]
    type InnerClass;
    #[wasm_bindgen(js_namespace = ["nestedNamespace", "InnerClass"])]
    fn inner_static_function(a: u32) -> u32;
    #[wasm_bindgen(js_namespace = ["nestedNamespace", "InnerClass"])]
    fn create_inner_instance() -> InnerClass;
    #[wasm_bindgen(method)]
    fn get_internal_int(this: &InnerClass) -> u32;
    #[wasm_bindgen(method)]
    fn append_to_internal_int(this: &InnerClass, i: u32);
    #[wasm_bindgen(method)]
    fn assert_internal_int(this: &InnerClass, i: u32);
}

#[wasm_bindgen(js_namespace = Math)]
extern "C" {
    #[wasm_bindgen]
    fn random() -> f64;
    #[wasm_bindgen]
    fn log(a: f64) -> f64;
}

#[wasm_bindgen_test]
fn simple() {
    random();
    assert_eq!(log(1.0), math_log(1.0));
}

#[wasm_bindgen_test]
fn import_class() {
    assert_eq!(bar(), 2);
}

#[wasm_bindgen_test]
fn construct() {
    let f = Construct::create();
    assert_eq!(f.get_internal_string(), "this");
    assert_eq!(f.clone().get_internal_string(), "this");
    f.append_to_internal_string(" foo");
    f.assert_internal_string("this foo");

    assert_eq!(f.kebab_case(), 42);
    f.set_kebab_case_val(0);
    // our setter does nothing so this is 42 anyway
    assert_eq!(f.kebab_case_val(), 42);
    assert_eq!(Construct::static_kebab_case(), 42);
    Construct::set_static_kebab_case_val(0);
    assert_eq!(Construct::static_kebab_case_val(), 42);
}

#[wasm_bindgen_test]
fn new_constructors() {
    let f = NewConstructors::new(1);
    assert_eq!(f.get(), 2);
}

#[wasm_bindgen_test]
fn rename_type() {
    let f = RenamedTypes::new(1);
    assert_eq!(f.get(), 2);
}

#[wasm_bindgen_test]
fn switch_methods() {
    assert!(!switch_methods_called());
    SwitchMethods::a();
    assert!(switch_methods_called());

    switch_methods_a();

    assert!(!switch_methods_called());
    SwitchMethods::a();
    assert!(switch_methods_called());

    assert!(!switch_methods_called());
    SwitchMethods::new().b();
    assert!(switch_methods_called());

    switch_methods_b();

    assert!(!switch_methods_called());
    SwitchMethods::new().b();
    assert!(!switch_methods_called());
}

#[wasm_bindgen_test]
fn properties() {
    let a = Properties::new();
    assert_eq!(a.a(), 1);
    a.set_a(2);
    assert_eq!(a.a(), 2);
}

#[wasm_bindgen_test]
fn rename_setter_getter() {
    let x: fn() -> RenameProperties = RenameProperties::new;
    let a = x();
    assert_eq!(a.test(), 1);
    a.another(2);
    assert_eq!(a.test(), 2);
    a.another2(3);
    assert_eq!(a.test2(), 3);
}

/// dox
#[wasm_bindgen]
pub struct AssertDenyDocsWorks {
    /// dox
    pub a: u32,
    _b: i64,
}

/// dox
#[wasm_bindgen]
pub fn assert_deny_docs_works() {}

#[wasm_bindgen_test]
fn options() {
    take_none(None);
    take_some(Some(Options::new()));
    assert!(return_null().is_none());
    assert!(return_undefined().is_none());
    assert!(return_some().is_some());
    run_rust_option_tests();
}

/// doc
#[wasm_bindgen]
pub fn rust_take_none(a: Option<Options>) {
    assert!(a.is_none());
}

/// doc
#[wasm_bindgen]
pub fn rust_take_some(a: Option<Options>) {
    assert!(a.is_some());
}

/// doc
#[wasm_bindgen]
pub fn rust_return_none() -> Option<Options> {
    None
}

/// doc
#[wasm_bindgen]
pub fn rust_return_some() -> Option<Options> {
    Some(Options::new())
}

#[wasm_bindgen_test]
fn catch_constructors() {
    assert!(CatchConstructors::new(0).is_err());
    assert!(CatchConstructors::new(1).is_ok());
}

#[wasm_bindgen_test]
fn static_structural() {
    assert_eq!(StaticStructural::static_structural(30), 33);
}

#[wasm_bindgen_test]
fn nested_namespace() {
    assert_eq!(InnerClass::inner_static_function(15), 20);

    let f = InnerClass::create_inner_instance();
    assert_eq!(f.get_internal_int(), 3);
    assert_eq!(f.clone().get_internal_int(), 3);
    f.append_to_internal_int(5);
    f.assert_internal_int(8);
}
