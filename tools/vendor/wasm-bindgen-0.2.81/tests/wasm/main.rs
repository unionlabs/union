#![cfg(target_arch = "wasm32")]

extern crate js_sys;
extern crate wasm_bindgen;
extern crate wasm_bindgen_test;
extern crate wasm_bindgen_test_crate_a;
extern crate wasm_bindgen_test_crate_b;

#[cfg(feature = "serde-serialize")]
#[macro_use]
extern crate serde_derive;

use wasm_bindgen::prelude::*;

pub mod api;
pub mod arg_names;
pub mod char;
pub mod classes;
pub mod closures;
pub mod comments;
pub mod duplicate_deps;
pub mod duplicates;
pub mod enums;
#[path = "final.rs"]
pub mod final_;
pub mod futures;
pub mod getters_and_setters;
pub mod import_class;
pub mod imports;
pub mod js_objects;
pub mod jscast;
pub mod math;
pub mod no_shims;
pub mod node;
pub mod option;
pub mod optional_primitives;
pub mod result;
pub mod result_jserror;
pub mod rethrow;
pub mod simple;
pub mod slice;
pub mod structural;
pub mod truthy_falsy;
pub mod u64;
pub mod validate_prt;
pub mod variadic;
pub mod vendor_prefix;

// should not be executed
#[wasm_bindgen(start)]
pub fn start() {
    panic!();
}
