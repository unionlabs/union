#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `PcImplIceGatheringState` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `PcImplIceGatheringState`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcImplIceGatheringState {
    New = "new",
    Gathering = "gathering",
    Complete = "complete",
}
