use js_sys::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn clear() {
    let map = Map::new();
    map.set(&"foo".into(), &"bar".into());
    map.set(&"bar".into(), &"baz".into());
    assert_eq!(map.size(), 2);
    map.clear();
    assert_eq!(map.size(), 0);
    map.clear();
    assert_eq!(map.size(), 0);
}

#[wasm_bindgen_test]
fn delete() {
    let map = Map::new();
    map.set(&"foo".into(), &"bar".into());
    assert_eq!(map.size(), 1);
    assert_eq!(map.delete(&"foo".into()), true);
    assert_eq!(map.delete(&"bar".into()), false);
    assert_eq!(map.size(), 0);
}

#[wasm_bindgen_test]
fn for_each() {
    let map = Map::new();
    map.set(&1.into(), &true.into());
    map.set(&2.into(), &false.into());
    map.set(&3.into(), &"awoo".into());
    map.set(&4.into(), &100.into());
    map.set(&5.into(), &Array::new().into());
    map.set(&6.into(), &Object::new().into());

    let mut res = Vec::new();
    map.for_each(&mut |value, key| {
        if value.as_bool().is_some() {
            res.push((key, value));
        }
    });

    assert_eq!(map.size(), 6);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].0, 1);
    assert_eq!(res[0].1, true);
    assert_eq!(res[1].0, 2);
    assert_eq!(res[1].1, false);
}

#[wasm_bindgen_test]
fn get() {
    let map = Map::new();
    map.set(&"foo".into(), &"bar".into());
    map.set(&1.into(), &2.into());
    assert_eq!(map.get(&"foo".into()), "bar");
    assert_eq!(map.get(&1.into()), 2);
    assert!(map.get(&2.into()).is_undefined());
}

#[wasm_bindgen_test]
fn has() {
    let map = Map::new();
    map.set(&"foo".into(), &"bar".into());
    assert_eq!(map.has(&"foo".into()), true);
    assert_eq!(map.has(&"bar".into()), false);
}

#[wasm_bindgen_test]
fn new() {
    assert_eq!(Map::new().size(), 0);
}

#[wasm_bindgen_test]
fn set() {
    let map = Map::new();
    let new = map.set(&"foo".into(), &"bar".into());
    assert_eq!(map.has(&"foo".into()), true);
    assert_eq!(new.has(&"foo".into()), true);
}

#[wasm_bindgen_test]
fn size() {
    let map = Map::new();
    map.set(&"foo".into(), &"bar".into());
    map.set(&"bar".into(), &"baz".into());
    assert_eq!(map.size(), 2);
}

#[wasm_bindgen_test]
fn map_inheritance() {
    let map = Map::new();
    assert!(map.is_instance_of::<Map>());
    assert!(map.is_instance_of::<Object>());
    let _: &Object = map.as_ref();
}
