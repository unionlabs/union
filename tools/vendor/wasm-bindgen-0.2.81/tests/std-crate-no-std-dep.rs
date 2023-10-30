//! This is a test that we can define items in a `#![no_std]` crate when
//! `wasm-bindgen` is compiled itself with the `std` feature and everything
//! works out just fine.

#![no_std]

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn test(a: &str);

    type Js;
    #[wasm_bindgen(constructor)]
    fn new() -> Js;
    #[wasm_bindgen(method, structural)]
    fn init(this: &Js);
}

#[wasm_bindgen]
pub struct A {}

#[wasm_bindgen]
impl A {
    pub fn foo(&self) {}
    pub fn bar(&mut self) {}
}
