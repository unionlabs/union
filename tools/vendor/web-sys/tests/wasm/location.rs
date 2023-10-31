use wasm_bindgen_test::*;
use web_sys::{self, Location};

fn location() -> Location {
    web_sys::window().unwrap().location()
}

#[wasm_bindgen_test]
fn href() {
    let loc = location();
    loc.href().unwrap();
}

#[wasm_bindgen_test]
fn origin() {
    let loc = location();
    loc.origin().unwrap();
}

#[wasm_bindgen_test]
fn protocol() {
    let loc = location();
    loc.protocol().unwrap();
}

#[wasm_bindgen_test]
fn host() {
    let loc = location();
    loc.host().unwrap();
}

#[wasm_bindgen_test]
fn hostname() {
    let loc = location();
    loc.hostname().unwrap();
}

#[wasm_bindgen_test]
fn port() {
    let loc = location();
    loc.port().unwrap();
}

#[wasm_bindgen_test]
fn pathname() {
    let loc = location();
    loc.pathname().unwrap();
}

#[wasm_bindgen_test]
fn search() {
    let loc = location();
    loc.search().unwrap();
}

#[wasm_bindgen_test]
fn hash() {
    let loc = location();
    loc.hash().unwrap();
}
