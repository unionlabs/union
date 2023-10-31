#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `GridTrackState` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `GridTrackState`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridTrackState {
    Static = "static",
    Repeat = "repeat",
    Removed = "removed",
}
