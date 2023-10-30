use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/slice.js")]
extern "C" {
    fn js_export();

    fn js_import();

    fn js_pass_array();

    fn js_export_mut();

    fn js_return_vec();

    fn js_clamped(val: Clamped<&[u8]>, offset: u8);
    #[wasm_bindgen(js_name = js_clamped)]
    fn js_clamped2(val: Clamped<Vec<u8>>, offset: u8);
    #[wasm_bindgen(js_name = js_clamped)]
    fn js_clamped3(val: Clamped<&mut [u8]>, offset: u8);
}

macro_rules! export_macro {
    ($(($i:ident, $n:ident))*) => ($(
        #[wasm_bindgen]
        pub fn $n(a: &[$i]) -> Vec<$i> {
            assert_eq!(a.len(), 2);
            assert_eq!(a[0], 1 as $i);
            assert_eq!(a[1], 2 as $i);
            a.to_vec()
        }
    )*)
}

export_macro! {
    (i8, export_i8)
    (u8, export_u8)
    (i16, export_i16)
    (u16, export_u16)
    (i32, export_i32)
    (u32, export_u32)
    (isize, export_isize)
    (usize, export_usize)
    (f32, export_f32)
    (f64, export_f64)
}

#[wasm_bindgen_test]
fn export() {
    js_export();
}

macro_rules! import_macro {
    ($(($rust:ident, $js:ident, $i:ident))*) => ($(
        #[wasm_bindgen(module = "tests/wasm/slice.js")]
        extern "C" {
            fn $js(a: &[$i], b: Option<&[$i]>, c: Option<&[$i]>) -> Vec<$i>;
        }

        #[wasm_bindgen]
        pub fn $rust(a: &[$i]) -> Vec<$i> {
            assert_eq!(a.len(), 2);
            assert_eq!(a[0], 1 as $i);
            assert_eq!(a[1], 2 as $i);
            $js(a, Some(a), None)
        }
    )*)
}

import_macro! {
    (import_rust_i8, import_js_i8, i8)
    (import_rust_u8, import_js_u8, u8)
    (import_rust_i16, import_js_i16, i16)
    (import_rust_u16, import_js_u16, u16)
    (import_rust_i32, import_js_i32, i32)
    (import_rust_u32, import_js_u32, u32)
    (import_rust_isize, import_js_isize, isize)
    (import_rust_usize, import_js_usize, usize)
    (import_rust_f32, import_js_f32, f32)
    (import_rust_f64, import_js_f64, f64)
}

#[wasm_bindgen_test]
fn import() {
    js_import();
}

macro_rules! pass_array_marco {
    ($(($rust:ident, $i:ident))*) => ($(
        #[wasm_bindgen]
        pub fn $rust(a: &[$i]) {
            assert_eq!(a.len(), 2);
            assert_eq!(a[0], 1 as $i);
            assert_eq!(a[1], 2 as $i);
        }
    )*)
}

pass_array_marco! {
    (pass_array_rust_i8, i8)
    (pass_array_rust_u8, u8)
    (pass_array_rust_i16, i16)
    (pass_array_rust_u16, u16)
    (pass_array_rust_i32, i32)
    (pass_array_rust_u32, u32)
    (pass_array_rust_isize, isize)
    (pass_array_rust_usize, usize)
    (pass_array_rust_f32, f32)
    (pass_array_rust_f64, f64)
}

#[wasm_bindgen_test]
fn pass_array() {
    js_pass_array();
}

macro_rules! import_mut_macro {
    ($(($rust:ident, $js:ident, $i:ident))*) => (
        $(
            #[wasm_bindgen(module = "tests/wasm/slice.js")]
            extern "C" {
                fn $js(a: &mut [$i], b: Option<&mut [$i]>, c: Option<&mut [$i]>);
            }

            fn $rust() {
                let mut buf1 = [
                    1 as $i,
                    2 as $i,
                    3 as $i,
                ];
                let mut buf2 = [
                    4 as $i,
                    5 as $i,
                    6 as $i,
                ];
                $js(&mut buf1, Some(&mut buf2), None);
                assert_eq!(buf1[0], 4 as $i);
                assert_eq!(buf1[1], 5 as $i);
                assert_eq!(buf1[2], 3 as $i);
                assert_eq!(buf2[0], 8 as $i);
                assert_eq!(buf2[1], 7 as $i);
                assert_eq!(buf2[2], 6 as $i);
            }
        )*

        #[wasm_bindgen_test]
        fn import_mut() {
            $($rust();)*
        }
    )
}

import_mut_macro! {
    (import_mut_rust_i8, import_mut_js_i8, i8)
    (import_mut_rust_u8, import_mut_js_u8, u8)
    (import_mut_rust_i16, import_mut_js_i16, i16)
    (import_mut_rust_u16, import_mut_js_u16, u16)
    (import_mut_rust_i32, import_mut_js_i32, i32)
    (import_mut_rust_u32, import_mut_js_u32, u32)
    (import_mut_rust_f32, import_mut_js_f32, f32)
    (import_mut_rust_f64, import_mut_js_f64, f64)
}

macro_rules! export_mut_macro {
    ($(($i:ident, $n:ident))*) => ($(
        #[wasm_bindgen]
        pub fn $n(a: &mut [$i])  {
            assert_eq!(a.len(), 3);
            assert_eq!(a[0], 1 as $i);
            assert_eq!(a[1], 2 as $i);
            assert_eq!(a[2], 3 as $i);
            a[0] = 4 as $i;
            a[1] = 5 as $i;
        }
    )*)
}

export_mut_macro! {
    (i8, export_mut_i8)
    (u8, export_mut_u8)
    (i16, export_mut_i16)
    (u16, export_mut_u16)
    (i32, export_mut_i32)
    (u32, export_mut_u32)
    (isize, export_mut_isize)
    (usize, export_mut_usize)
    (f32, export_mut_f32)
    (f64, export_mut_f64)
}

#[wasm_bindgen_test]
fn export_mut() {
    js_export_mut();
}

#[wasm_bindgen]
pub fn return_vec_broken_vec() -> Vec<u32> {
    vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
}

#[wasm_bindgen]
pub fn return_vec_web_main() -> ReturnVecApplication {
    ReturnVecApplication::new()
}

#[wasm_bindgen]
pub struct ReturnVecApplication {
    thing: Vec<u32>,
}

#[wasm_bindgen]
impl ReturnVecApplication {
    #[allow(clippy::vec_init_then_push)]
    pub fn new() -> ReturnVecApplication {
        let mut thing = vec![];
        thing.push(0);
        thing.push(0);
        thing.push(0);
        thing.push(0);
        thing.push(0);

        ReturnVecApplication { thing }
    }

    pub fn tick(&mut self) {
        self.thing = self.thing.clone();
    }
}

#[wasm_bindgen_test]
fn return_vec() {
    js_return_vec();
}

#[wasm_bindgen_test]
fn take_clamped() {
    js_clamped(Clamped(&[1, 2, 3]), 1);
    js_clamped2(Clamped(vec![4, 5, 6]), 4);
    js_clamped3(Clamped(&mut [7, 8, 9]), 7);
}
