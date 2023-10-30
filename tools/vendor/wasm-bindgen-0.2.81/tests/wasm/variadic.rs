use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

#[wasm_bindgen(module = "tests/wasm/variadic.js")]
extern "C" {
    #[wasm_bindgen(variadic)]
    fn variadic_sum_u8(first: u8, second: u8, rest: &[u8]) -> u8;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_u16(first: u16, second: u16, rest: &[u16]) -> u16;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_u32(first: u32, second: u32, rest: &[u32]) -> u32;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_u64(first: u64, second: u64, rest: &[u64]) -> u64;
    //#[wasm_bindgen(variadic)]
    //fn variadic_sum_usize(first: usize, second: usize, rest: &[usize]) -> usize;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_i8(first: i8, second: i8, rest: &[i8]) -> i8;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_i16(first: i16, second: i16, rest: &[i16]) -> i16;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_i32(first: i32, second: i32, rest: &[i32]) -> i32;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_i64(first: i64, second: i64, rest: &[i64]) -> i64;
    //#[wasm_bindgen(variadic)]
    //fn variadic_sum_isize(first: isize, second: isize, rest: &[isize]) -> isize;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_f32(first: f32, second: f32, rest: &[f32]) -> f32;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_f64(first: f64, second: f64, rest: &[f64]) -> f64;
    //#[wasm_bindgen(variadic)]
    //fn variadic_sum_opt(first: Option<u32>, second: Option<u32>, rest: &[Option<u32>]) -> u32;
    //#[wasm_bindgen(variadic)]
    //fn variadic_concat_str(first: &str, second: &str, rest: &[&str]) -> String;
    //#[wasm_bindgen(variadic)]
    //fn variadic_concat_string(first: String,
    //                          second: String,
    //                          rest: Vec<String>) -> String;
    #[wasm_bindgen(variadic)]
    fn variadic_sum_rest_vec(first: u8, second: u8, rest: Vec<u8>) -> u8;
    //#[wasm_bindgen(variadic)]
    //fn variadic_compare_pairs(first: JsValue, second: JsValue, rest: &[JsValue]);
    //TODO imported type
}

// ints

macro_rules! variadic_test_int {
    ($fn_name:ident, $extern_name:ident) => {
        #[wasm_bindgen_test]
        fn $fn_name() {
            assert_eq!($extern_name(1, 2, &[]), 3);
            assert_eq!($extern_name(1, 2, &[3]), 6);
            assert_eq!($extern_name(1, 2, &[3, 4]), 10);
        }
    };
}

// The <int>64 tests throw js `Cannot mix BigInt and other types, use explicit conversions`
variadic_test_int!(u8, variadic_sum_u8);
variadic_test_int!(u16, variadic_sum_u16);
variadic_test_int!(u32, variadic_sum_u32);
//variadic_test_int!(u64, variadic_sum_u64);
//variadic_test_int!(usize, variadic_sum_usize);
variadic_test_int!(i8, variadic_sum_i8);
variadic_test_int!(i16, variadic_sum_i16);
variadic_test_int!(i32, variadic_sum_i32);
//variadic_test_int!(i64, variadic_sum_i64);
//variadic_test_int!(isize, variadic_sum_isize);

// floats

macro_rules! variadic_test_float {
    ($fn_name:ident, $extern_name:ident) => {
        #[wasm_bindgen_test]
        fn $fn_name() {
            assert_eq!($extern_name(1., 2., &[]), 3.);
            assert_eq!($extern_name(1., 2., &[3.]), 6.);
            assert_eq!($extern_name(1., 2., &[3., 4.]), 10.);
        }
    };
}

variadic_test_float!(f32, variadic_sum_f32);
variadic_test_float!(f64, variadic_sum_f64);

// strings

// `the trait `wasm_bindgen::convert::IntoWasmAbi` is not implemented for `&[&str]`
/*
#[wasm_bindgen_test]
fn str() {
    assert_eq!(variadic_concat_str("a ", "test", &[]), "a test");
    assert_eq!(variadic_concat_str("a", "nother ", &["test"]), "another test");
    assert_eq!(variadic_concat_str("yet ", "a", &["nother ", "test"]), "yet another test");
}

#[wasm_bindgen_test]
fn string() {
    assert_eq!(variadic_concat_string("a ".into(), "test".into(), vec![]), "a test");
    assert_eq!(variadic_concat_string("a".into(), "nother ".into(), vec!["test".into()]),
               "another test");
    assert_eq!(variadic_concat_string("yet ".into(),
                                      "a".into(),
                                      vec!["nother ".into(), "test".into()]),
               "yet another test");
}
*/

// options

/*
#[wasm_bindgen_test]
fn opt() {
    assert_eq!(variadic_sum_opt(Some(1), None, &[]), 1);
    assert_eq!(variadic_sum_opt(Some(1), None, &[Some(2)]), 3);
    assert_eq!(variadic_sum_opt(Some(1), None, &[None, Some(2)]), 3);
}
*/

// vec not slice

#[wasm_bindgen_test]
fn rest_vec() {
    assert_eq!(variadic_sum_rest_vec(1, 2, vec![]), 3);
    assert_eq!(variadic_sum_rest_vec(1, 2, vec![3]), 6);
    assert_eq!(variadic_sum_rest_vec(1, 2, vec![3, 4]), 10);
}

// JsValue
//#[wasm_bindgen_test]
//fn jsvalue() {
//    variadic_compare_pairs_jsvalue(true, true, vec![]);
//    variadic_compare_pairs_jsvalue(false, false, vec![3, 3]);
//}
