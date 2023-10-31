#[allow(unused_imports)] // test for #919
use std::borrow::BorrowMut;

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/classes.js")]
extern "C" {
    fn js_simple();
    fn js_strings();
    fn js_exceptions();
    fn js_pass_one_to_another();
    fn take_class(foo: ClassesIntoJs);
    #[wasm_bindgen(js_name = take_class)]
    fn take_class_as_jsvalue(foo: JsValue);
    fn js_constructors();
    fn js_empty_structs();
    fn js_public_fields();
    fn js_getter_with_clone();
    fn js_using_self();
    fn js_readonly_fields();
    fn js_double_consume();
    fn js_js_rename();
    fn js_access_fields();
    fn js_renamed_export();
    fn js_renamed_field();
    fn js_conditional_bindings();

    fn js_assert_none(a: Option<OptionClass>);
    fn js_assert_some(a: Option<OptionClass>);
    fn js_return_none1() -> Option<OptionClass>;
    fn js_return_none2() -> Option<OptionClass>;
    fn js_return_some(a: OptionClass) -> Option<OptionClass>;
    fn js_test_option_classes();
    fn js_test_inspectable_classes();
    fn js_test_inspectable_classes_can_override_generated_methods();
}

#[wasm_bindgen_test]
fn simple() {
    js_simple();
}

#[wasm_bindgen]
pub struct ClassesSimple {
    contents: u32,
}

#[wasm_bindgen]
impl ClassesSimple {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ClassesSimple {
        ClassesSimple::with_contents(0)
    }

    pub fn with_contents(a: u32) -> ClassesSimple {
        ClassesSimple { contents: a }
    }

    pub fn add(&mut self, amt: u32) -> u32 {
        self.contents += amt;
        self.contents
    }

    pub fn consume(self) -> u32 {
        self.contents
    }
}

#[wasm_bindgen_test]
fn strings() {
    js_strings()
}

#[wasm_bindgen]
pub struct ClassesStrings1 {
    name: u32,
}

#[wasm_bindgen]
pub struct ClassesStrings2 {
    contents: String,
}

#[wasm_bindgen]
impl ClassesStrings1 {
    pub fn new() -> ClassesStrings1 {
        ClassesStrings1 { name: 0 }
    }

    pub fn set(&mut self, amt: u32) {
        self.name = amt;
    }

    pub fn bar(&self, mix: &str) -> ClassesStrings2 {
        ClassesStrings2 {
            contents: format!("foo-{}-{}", mix, self.name),
        }
    }
}

#[wasm_bindgen]
impl ClassesStrings2 {
    pub fn name(&self) -> String {
        self.contents.clone()
    }
}

#[wasm_bindgen_test]
fn exceptions() {
    js_exceptions();
}

#[wasm_bindgen]
pub struct ClassesExceptions1 {}

#[wasm_bindgen]
impl ClassesExceptions1 {
    pub fn new() -> ClassesExceptions1 {
        ClassesExceptions1 {}
    }

    pub fn foo(&self, _: &ClassesExceptions1) {}

    pub fn bar(&mut self, _: &mut ClassesExceptions1) {}
}

#[wasm_bindgen]
pub struct ClassesExceptions2 {}

#[wasm_bindgen]
impl ClassesExceptions2 {
    pub fn new() -> ClassesExceptions2 {
        ClassesExceptions2 {}
    }
}

#[wasm_bindgen_test]
fn pass_one_to_another() {
    js_pass_one_to_another();
}

#[wasm_bindgen]
pub struct ClassesPassA {}

#[wasm_bindgen]
impl ClassesPassA {
    pub fn new() -> ClassesPassA {
        ClassesPassA {}
    }

    pub fn foo(&self, _other: &ClassesPassB) {}

    pub fn bar(&self, _other: ClassesPassB) {}
}

#[wasm_bindgen]
pub struct ClassesPassB {}

#[wasm_bindgen]
impl ClassesPassB {
    pub fn new() -> ClassesPassB {
        ClassesPassB {}
    }
}

#[wasm_bindgen_test]
fn pass_into_js() {
    take_class(ClassesIntoJs(13));
}

#[wasm_bindgen]
pub struct ClassesIntoJs(i32);

#[wasm_bindgen]
impl ClassesIntoJs {
    pub fn inner(&self) -> i32 {
        self.0
    }
}

#[wasm_bindgen]
pub struct Issue27Context {}

#[wasm_bindgen]
impl Issue27Context {
    pub fn parse(&self, _expr: &str) -> Issue27Expr {
        panic!()
    }
    pub fn eval(&self, _expr: &Issue27Expr) -> f64 {
        panic!()
    }
    pub fn set(&mut self, _var: &str, _val: f64) {
        panic!()
    }
}

#[wasm_bindgen]
pub struct Issue27Expr {}

#[wasm_bindgen_test]
fn pass_into_js_as_js_class() {
    take_class_as_jsvalue(ClassesIntoJs(13).into());
}

#[wasm_bindgen_test]
fn constructors() {
    js_constructors();
}

#[wasm_bindgen]
pub fn cross_item_construction() -> ConstructorsBar {
    ConstructorsBar::other_name(7, 8)
}

#[wasm_bindgen]
pub struct ConstructorsFoo {
    number: u32,
}

#[wasm_bindgen]
impl ConstructorsFoo {
    #[wasm_bindgen(constructor)]
    pub fn new(number: u32) -> ConstructorsFoo {
        ConstructorsFoo { number }
    }

    pub fn get_number(&self) -> u32 {
        self.number
    }
}

#[wasm_bindgen]
pub struct ConstructorsBar {
    number: u32,
    number2: u32,
}

#[wasm_bindgen]
impl ConstructorsBar {
    #[wasm_bindgen(constructor)]
    pub fn other_name(number: u32, number2: u32) -> ConstructorsBar {
        ConstructorsBar { number, number2 }
    }

    pub fn get_sum(&self) -> u32 {
        self.number + self.number2
    }
}

#[wasm_bindgen_test]
fn empty_structs() {
    js_empty_structs();
}

#[wasm_bindgen]
pub struct MissingClass {}

#[wasm_bindgen]
pub struct OtherEmpty {}

#[wasm_bindgen]
impl OtherEmpty {
    pub fn return_a_value() -> MissingClass {
        MissingClass {}
    }
}

#[wasm_bindgen_test]
fn public_fields() {
    js_public_fields();
}

#[wasm_bindgen]
#[derive(Default)]
pub struct PublicFields {
    pub a: u32,
    pub b: f32,
    pub c: f64,
    pub d: i32,
    #[wasm_bindgen(skip)]
    pub skipped: u32,
}

#[wasm_bindgen]
impl PublicFields {
    pub fn new() -> PublicFields {
        PublicFields::default()
    }
}

#[wasm_bindgen_test]
fn getter_with_clone() {
    js_getter_with_clone();
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default)]
pub struct GetterWithCloneStruct {
    pub a: String,
}

#[wasm_bindgen]
impl GetterWithCloneStruct {
    pub fn new() -> GetterWithCloneStruct {
        GetterWithCloneStruct::default()
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct GetterWithCloneStructField {
    #[wasm_bindgen(getter_with_clone)]
    pub a: String,
}

#[wasm_bindgen]
impl GetterWithCloneStructField {
    pub fn new() -> GetterWithCloneStructField {
        GetterWithCloneStructField::default()
    }
}

#[wasm_bindgen_test]
fn using_self() {
    js_using_self();
}

#[wasm_bindgen]
pub struct UseSelf {}

#[wasm_bindgen]
impl UseSelf {
    pub fn new() -> Self {
        UseSelf {}
    }
}

#[wasm_bindgen_test]
fn readonly_fields() {
    js_readonly_fields();
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Readonly {
    #[wasm_bindgen(readonly)]
    pub a: u32,
}

#[wasm_bindgen]
impl Readonly {
    pub fn new() -> Readonly {
        Readonly::default()
    }
}

#[wasm_bindgen_test]
fn double_consume() {
    js_double_consume();
}

#[wasm_bindgen]
pub struct DoubleConsume {}

#[wasm_bindgen]
impl DoubleConsume {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DoubleConsume {
        DoubleConsume {}
    }

    pub fn consume(self, other: DoubleConsume) {
        drop(other);
    }
}

#[wasm_bindgen_test]
fn rename_function_for_js() {
    js_js_rename();
    foo();
}

#[wasm_bindgen]
pub struct JsRename {}

#[wasm_bindgen]
impl JsRename {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsRename {
        let f = JsRename {};
        f.foo();
        f
    }

    #[wasm_bindgen(js_name = bar)]
    pub fn foo(&self) {}
}

#[wasm_bindgen(js_name = classes_foo)]
pub fn foo() {}

#[wasm_bindgen]
pub struct AccessFieldFoo {
    pub bar: AccessFieldBar,
}

#[wasm_bindgen]
pub struct AccessField0(pub AccessFieldBar);

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct AccessFieldBar {
    _value: u32,
}

#[wasm_bindgen]
impl AccessFieldFoo {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AccessFieldFoo {
        AccessFieldFoo {
            bar: AccessFieldBar { _value: 2 },
        }
    }
}

#[wasm_bindgen]
impl AccessField0 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AccessField0 {
        AccessField0(AccessFieldBar { _value: 2 })
    }
}

#[wasm_bindgen_test]
fn access_fields() {
    js_access_fields();
}

#[wasm_bindgen(js_name = JsRenamedExport)]
pub struct RenamedExport {
    pub x: u32,
}

#[wasm_bindgen(js_class = JsRenamedExport)]
impl RenamedExport {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RenamedExport {
        RenamedExport { x: 3 }
    }
    pub fn foo(&self) {}

    pub fn bar(&self, other: &RenamedExport) {
        drop(other);
    }
}

#[wasm_bindgen_test]
fn renamed_export() {
    js_renamed_export();
}

#[wasm_bindgen]
pub struct RenamedField {
    #[wasm_bindgen(js_name = bar)]
    pub foo: u32,
}

#[wasm_bindgen(js_class = RenamedField)]
impl RenamedField {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RenamedField {
        RenamedField { foo: 3 }
    }

    pub fn foo(&self) {}
}

#[wasm_bindgen_test]
fn renamed_field() {
    js_renamed_field();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct ConditionalBindings {}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl ConditionalBindings {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> ConditionalBindings {
        ConditionalBindings {}
    }
}

#[wasm_bindgen_test]
fn conditional_bindings() {
    js_conditional_bindings();
}

#[wasm_bindgen]
pub struct OptionClass(u32);

#[wasm_bindgen_test]
fn option_class() {
    js_assert_none(None);
    js_assert_some(Some(OptionClass(1)));
    assert!(js_return_none1().is_none());
    assert!(js_return_none2().is_none());
    assert_eq!(js_return_some(OptionClass(2)).unwrap().0, 2);
    js_test_option_classes();
}

#[wasm_bindgen]
pub fn option_class_none() -> Option<OptionClass> {
    None
}

#[wasm_bindgen]
pub fn option_class_some() -> Option<OptionClass> {
    Some(OptionClass(3))
}

#[wasm_bindgen]
pub fn option_class_assert_none(x: Option<OptionClass>) {
    assert!(x.is_none());
}

#[wasm_bindgen]
pub fn option_class_assert_some(x: Option<OptionClass>) {
    assert_eq!(x.unwrap().0, 3);
}

mod works_in_module {
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub struct WorksInModule(u32);

    #[wasm_bindgen]
    impl WorksInModule {
        #[wasm_bindgen(constructor)]
        pub fn new() -> WorksInModule {
            WorksInModule(1)
        }

        pub fn foo(&self) {}
    }
}

#[wasm_bindgen_test]
fn inspectable_classes() {
    js_test_inspectable_classes();
}

#[wasm_bindgen(inspectable)]
#[derive(Default)]
pub struct Inspectable {
    pub a: u32,
    // This private field will not be exposed unless a getter is provided for it
    #[allow(dead_code)]
    private: u32,
}

#[wasm_bindgen]
impl Inspectable {
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct NotInspectable {
    pub a: u32,
}

#[wasm_bindgen]
impl NotInspectable {
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen_test]
fn inspectable_classes_can_override_generated_methods() {
    js_test_inspectable_classes_can_override_generated_methods();
}

#[wasm_bindgen(inspectable)]
#[derive(Default)]
pub struct OverriddenInspectable {
    pub a: u32,
}

#[wasm_bindgen]
impl OverriddenInspectable {
    pub fn new() -> Self {
        Self::default()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn js_to_json(&self) -> String {
        String::from("JSON was overwritten")
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn js_to_string(&self) -> String {
        String::from("string was overwritten")
    }
}
