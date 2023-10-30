#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `MediaStreamTrackState` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `MediaStreamTrackState`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaStreamTrackState {
    Live = "live",
    Ended = "ended",
}
