use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/Symbol.js")]
extern "C" {
    fn test_has_instance(sym: &Symbol);
    fn test_is_concat_spreadable(sym: &Symbol);
    fn test_iterator(sym: &Symbol);
    fn test_async_iterator(sym: &Symbol) -> Promise;
    fn test_match(sym: &Symbol);
    fn test_replace(sym: &Symbol);
    fn test_search(sym: &Symbol);
    fn test_species(sym: &Symbol);
    fn test_split(sym: &Symbol);
    fn test_to_primitive(sym: &Symbol);
    fn test_to_string_tag(sym: &Symbol);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Symbol)]
    fn gensym(val: JsValue) -> Symbol;
}

#[wasm_bindgen_test]
fn has_instance() {
    test_has_instance(&Symbol::has_instance());
}

#[wasm_bindgen_test]
fn is_concat_spreadable() {
    test_is_concat_spreadable(&Symbol::is_concat_spreadable());
}

#[wasm_bindgen_test]
fn iterator() {
    test_iterator(&Symbol::iterator());
}

#[wasm_bindgen_test]
async fn async_iterator() {
    JsFuture::from(test_async_iterator(&Symbol::async_iterator()))
        .await
        .unwrap_throw();
}

#[wasm_bindgen_test]
fn match_() {
    test_match(&Symbol::match_());
}

#[wasm_bindgen_test]
fn replace() {
    test_replace(&Symbol::replace());
}

#[wasm_bindgen_test]
fn search() {
    test_search(&Symbol::search());
}

#[wasm_bindgen_test]
fn species() {
    test_species(&Symbol::species());
}

#[wasm_bindgen_test]
fn split() {
    test_split(&Symbol::split());
}

#[wasm_bindgen_test]
fn to_primitive() {
    test_to_primitive(&Symbol::to_primitive());
}

#[wasm_bindgen_test]
fn to_string_tag() {
    test_to_string_tag(&Symbol::to_string_tag());
}

#[wasm_bindgen_test]
fn for_() {
    let foo = JsValue::from(Symbol::for_("foo"));
    let bar = JsValue::from(Symbol::for_("bar"));
    assert_eq!(foo, foo);
    assert_eq!(bar, bar);
    assert_ne!(foo, bar);
    assert_ne!(bar, foo);

    assert_eq!(Symbol::for_("mario").to_string(), "Symbol(mario)");
}

#[wasm_bindgen_test]
fn key_for() {
    let sym = Symbol::for_("foo");
    assert_eq!(Symbol::key_for(&sym), "foo");
    assert!(Symbol::key_for(&Symbol::iterator()).is_undefined());
    assert!(Symbol::key_for(&Symbol::async_iterator()).is_undefined());
    assert!(Symbol::key_for(&gensym(JsValue::undefined())).is_undefined());
}

#[wasm_bindgen_test]
fn to_string() {
    assert_eq!(Symbol::iterator().to_string(), "Symbol(Symbol.iterator)");
    assert_eq!(
        Symbol::async_iterator().to_string(),
        "Symbol(Symbol.asyncIterator)"
    );
    assert_eq!(Symbol::for_("foo").to_string(), "Symbol(foo)");
    assert_eq!(gensym("desc".into()).to_string(), "Symbol(desc)");
}

#[wasm_bindgen_test]
fn unscopables() {
    assert_eq!(
        Symbol::unscopables().to_string(),
        "Symbol(Symbol.unscopables)"
    );
}

#[wasm_bindgen_test]
fn value_of() {
    let a = Symbol::for_("foo");
    assert_eq!(JsValue::from(a.value_of()), JsValue::from(a));
    let a = gensym(JsValue::undefined());
    assert_eq!(JsValue::from(a.value_of()), JsValue::from(a));
}
