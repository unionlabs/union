use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

macro_rules! each {
    ($m:ident) => {
        $m!(Uint8Array);
        $m!(Uint8ClampedArray);
        $m!(Uint16Array);
        $m!(Uint32Array);
        $m!(Int8Array);
        $m!(Int16Array);
        $m!(Int32Array);
        $m!(Float32Array);
        $m!(Float64Array);
    };
}

macro_rules! test_inheritance {
    ($arr:ident) => {{
        let arr = $arr::new(&JsValue::undefined());
        assert!(arr.is_instance_of::<$arr>());
        let _: &Object = arr.as_ref();
        assert!(arr.is_instance_of::<Object>());
    }};
}
#[wasm_bindgen_test]
fn inheritance() {
    each!(test_inheritance);
}

macro_rules! test_undefined {
    ($arr:ident) => {{
        let arr = $arr::new(&JsValue::undefined());
        assert_eq!(arr.length(), 0);
        assert_eq!(arr.byte_length(), 0);
        assert_eq!(arr.byte_offset(), 0);
        assert!(JsValue::from(arr.buffer()).is_object());
    }};
}
#[wasm_bindgen_test]
fn new_undefined() {
    each!(test_undefined);
}

macro_rules! test_length {
    ($arr:ident) => {{
        let arr = $arr::new(&4.into());
        assert_eq!(arr.length(), 4);
        assert!(arr.byte_length() != 0);
        assert_eq!(arr.byte_offset(), 0);
        assert!(JsValue::from(arr.buffer()).is_object());
    }};
}
#[wasm_bindgen_test]
fn new_length() {
    each!(test_length);
}

macro_rules! test_subarray {
    ($arr:ident) => {{
        assert_eq!($arr::new(&4.into()).subarray(0, 1).length(), 1);
    }};
}
#[wasm_bindgen_test]
fn new_subarray() {
    each!(test_subarray);
}

macro_rules! test_fill {
    ($arr:ident) => {{
        let arr = $arr::new(&4.into());
        arr.for_each(&mut |x, _, _| {
            assert_eq!(x as f64, 0.0);
        });
        arr.fill(2 as _, 0, 2);
        arr.for_each(&mut |x, i, _| {
            if i < 2 {
                assert_eq!(x as f64, 2.0);
            } else {
                assert_eq!(x as f64, 0.0);
            }
        });
    }};
}
#[wasm_bindgen_test]
fn new_fill() {
    each!(test_fill);
}

macro_rules! test_at {
    ($arr:ident) => {{
        let arr = $arr::new(&2.into());
        arr.set_index(1, 1 as _);
        assert_eq!(arr.at(-1).unwrap() as f64, 1 as f64);
    }};
}
#[wasm_bindgen_test]
fn new_at() {
    each!(test_at);
}

macro_rules! test_get_set {
    ($arr:ident) => {{
        let arr = $arr::new(&1.into());
        assert_eq!(arr.get_index(0) as f64, 0 as f64);
        arr.set_index(0, 1 as _);
        assert_eq!(arr.get_index(0) as f64, 1 as f64);
    }};
}
#[wasm_bindgen_test]
fn new_get_set() {
    each!(test_get_set);
}

macro_rules! test_slice {
    ($arr:ident) => {{
        let arr = $arr::new(&4.into());
        assert_eq!(arr.length(), 4);
        assert_eq!(arr.slice(1, 2).length(), 1);
    }};
}
#[wasm_bindgen_test]
fn new_slice() {
    each!(test_slice);
}

#[wasm_bindgen_test]
fn view() {
    let x = [1, 2, 3];
    let array = unsafe { Int32Array::view(&x) };
    assert_eq!(array.length(), 3);
    array.for_each(&mut |x, i, _| {
        assert_eq!(x, (i + 1) as i32);
    });
}

#[wasm_bindgen_test]
fn from() {
    let x: Vec<i32> = vec![1, 2, 3];
    let array = Int32Array::from(x.as_slice());
    assert_eq!(array.length(), 3);
    array.for_each(&mut |x, i, _| {
        assert_eq!(x, (i + 1) as i32);
    });
}

#[wasm_bindgen_test]
fn copy_to() {
    let mut x = [0; 10];
    let array = Int32Array::new(&10.into());
    array.fill(5, 0, 10);
    array.copy_to(&mut x);
    for i in x.iter() {
        assert_eq!(*i, 5);
    }
}

#[wasm_bindgen_test]
fn copy_from() {
    let x = [1, 2, 3];
    let array = Int32Array::new(&3.into());
    array.copy_from(&x);
    array.for_each(&mut |x, i, _| {
        assert_eq!(x, (i + 1) as i32);
    });
}

#[wasm_bindgen_test]
fn to_vec() {
    let array = Int32Array::new(&10.into());
    array.fill(5, 0, 10);
    assert_eq!(array.to_vec(), vec![5, 5, 5, 5, 5, 5, 5, 5, 5, 5]);
}
