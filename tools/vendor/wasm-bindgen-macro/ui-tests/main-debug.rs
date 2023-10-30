use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(main)]
fn main() -> Result<(), Test> {
    unimplemented!()
}

struct Test;

impl fmt::Debug for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

#[wasm_bindgen(main)]
fn fail() {}
