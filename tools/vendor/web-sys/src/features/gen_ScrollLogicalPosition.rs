#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `ScrollLogicalPosition` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `ScrollLogicalPosition`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollLogicalPosition {
    Start = "start",
    Center = "center",
    End = "end",
    Nearest = "nearest",
}
