use js_sys::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn get_canonical_locales() {
    let locales = Array::new();
    locales.push(&"EN-US".into());
    locales.push(&"Fr".into());
    let locales = JsValue::from(locales);
    let canonical_locales = Intl::get_canonical_locales(&locales);
    assert_eq!(canonical_locales.length(), 2);
    canonical_locales.for_each(&mut |l, i, _| {
        if i == 0 {
            assert_eq!(l, "en-US");
        } else {
            assert_eq!(l, "fr");
        }
    });
    let canonical_locales = Intl::get_canonical_locales(&"EN-US".into());
    assert_eq!(canonical_locales.length(), 1);
    canonical_locales.for_each(&mut |l, _, _| {
        assert_eq!(l, "en-US");
    });
}

#[wasm_bindgen_test]
fn collator() {
    let locales = Array::of1(&JsValue::from("en-US"));
    let opts = Object::new();

    let c = Intl::Collator::new(&locales, &opts);
    assert!(c.compare().is_instance_of::<Function>());
    assert!(c.resolved_options().is_instance_of::<Object>());

    let a = Intl::Collator::supported_locales_of(&locales, &opts);
    assert!(a.is_instance_of::<Array>());
}

#[wasm_bindgen_test]
fn collator_inheritance() {
    let locales = Array::of1(&JsValue::from("en-US"));
    let opts = Object::new();
    let c = Intl::Collator::new(&locales, &opts);

    assert!(c.is_instance_of::<Intl::Collator>());
    assert!(c.is_instance_of::<Object>());
    let _: &Object = c.as_ref();
}

#[wasm_bindgen_test]
fn date_time_format() {
    let locales = Array::of1(&JsValue::from("en-US"));
    let opts = Object::new();
    let epoch = Date::new(&JsValue::from(0));

    let c = Intl::DateTimeFormat::new(&locales, &opts);
    assert!(c.format().is_instance_of::<Function>());
    assert!(c.format_to_parts(&epoch).is_instance_of::<Array>());
    assert!(c.resolved_options().is_instance_of::<Object>());

    let a = Intl::DateTimeFormat::supported_locales_of(&locales, &opts);
    assert!(a.is_instance_of::<Array>());
}

#[wasm_bindgen_test]
fn date_time_format_inheritance() {
    let locales = Array::of1(&JsValue::from("en-US"));
    let opts = Object::new();
    let c = Intl::DateTimeFormat::new(&locales, &opts);

    assert!(c.is_instance_of::<Intl::DateTimeFormat>());
    assert!(c.is_instance_of::<Object>());
    let _: &Object = c.as_ref();
}

#[wasm_bindgen_test]
fn number_format() {
    let locales = Array::of1(&JsValue::from("en-US"));
    let opts = Object::new();

    let n = Intl::NumberFormat::new(&locales, &opts);
    assert!(n.format().is_instance_of::<Function>());
    assert!(n.format_to_parts(42.5).is_instance_of::<Array>());
    assert!(n.resolved_options().is_instance_of::<Object>());

    let a = Intl::NumberFormat::supported_locales_of(&locales, &opts);
    assert!(a.is_instance_of::<Array>());
}

#[wasm_bindgen_test]
fn number_format_inheritance() {
    let locales = Array::of1(&JsValue::from("en-US"));
    let opts = Object::new();
    let n = Intl::NumberFormat::new(&locales, &opts);

    assert!(n.is_instance_of::<Intl::NumberFormat>());
    assert!(n.is_instance_of::<Object>());
    let _: &Object = n.as_ref();
}

#[wasm_bindgen_test]
fn plural_rules() {
    let locales = Array::of1(&JsValue::from("en-US"));
    let opts = Object::new();

    let r = Intl::PluralRules::new(&locales, &opts);
    assert!(r.resolved_options().is_instance_of::<Object>());
    assert_eq!(r.select(1_f64), "one");

    let a = Intl::PluralRules::supported_locales_of(&locales, &opts);
    assert!(a.is_instance_of::<Array>());
}

#[wasm_bindgen_test]
fn plural_rules_inheritance() {
    let locales = Array::of1(&JsValue::from("en-US"));
    let opts = Object::new();
    let r = Intl::PluralRules::new(&locales, &opts);

    assert!(r.is_instance_of::<Intl::PluralRules>());
    assert!(r.is_instance_of::<Object>());
    let _: &Object = r.as_ref();
}
