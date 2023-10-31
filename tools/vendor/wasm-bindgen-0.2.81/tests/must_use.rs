//! This test validates that the generated bindings don't cause linting warnings
//! when used with structs annotated with `#[must_use]`.

#![deny(unused)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[must_use]
pub struct MustUse {}
