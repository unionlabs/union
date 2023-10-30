extern crate wasm_bindgen as extern_test;

use wasm_bindgen::prelude::*;

mod test {
    pub use wasm_bindgen as test;
    pub use wasm_bindgen;
}

#[wasm_bindgen(wasm_bindgen = wasm_bindgen)]
pub fn good1() {}

#[wasm_bindgen(wasm_bindgen = ::wasm_bindgen)]
pub fn good2() {}

#[wasm_bindgen(wasm_bindgen = test::wasm_bindgen)]
pub fn good3() {}

#[wasm_bindgen(wasm_bindgen = test::test)]
pub fn good4() {}

#[wasm_bindgen(wasm_bindgen = extern_test)]
pub fn good5() {}

#[wasm_bindgen(wasm_bindgen_futures = wasm_bindgen_futures)]
pub fn good6() {}

#[wasm_bindgen(wasm_bindgen = wasm_bindgen)]
pub async fn good7() {}

#[wasm_bindgen(wasm_bindgen_futures = wasm_bindgen_futures)]
pub async fn good8() {}

#[wasm_bindgen(wasm_bindgen = wasm_bindgen, wasm_bindgen_futures = wasm_bindgen_futures)]
pub async fn good9() {}

#[wasm_bindgen(wasm_bindgen = test)]
pub fn bad1() {}

#[wasm_bindgen(wasm_bindgen_futures = test)]
pub async fn bad2() {}

fn main() {}
