use js_sys::*;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn entries() {
    let map = Map::new();
    map.set(&"uno".into(), &1.into());

    let entries = map.entries();

    let next = entries.next().unwrap();
    assert_eq!(next.done(), false);
    assert!(next.value().is_object());
    assert_eq!(Reflect::get(&next.value(), &0.into()).unwrap(), "uno");
    assert_eq!(Reflect::get(&next.value(), &1.into()).unwrap(), 1);

    let next = entries.next().unwrap();
    assert!(next.done());
    assert!(next.value().is_undefined());
}

#[wasm_bindgen_test]
fn keys() {
    let map = Map::new();
    map.set(&"uno".into(), &1.into());

    let keys = map.keys();

    let next = keys.next().unwrap();
    assert_eq!(next.done(), false);
    assert_eq!(next.value(), "uno");

    let next = keys.next().unwrap();
    assert!(next.done());
    assert!(next.value().is_undefined());
}

#[wasm_bindgen_test]
fn values() {
    let map = Map::new();
    map.set(&"uno".into(), &1.into());

    let values = map.values();

    let next = values.next().unwrap();
    assert_eq!(next.done(), false);
    assert_eq!(next.value(), 1);

    let next = values.next().unwrap();
    assert!(next.done());
    assert!(next.value().is_undefined());
}
