#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `SocketReadyState` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `SocketReadyState`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketReadyState {
    Opening = "opening",
    Open = "open",
    Closing = "closing",
    Closed = "closed",
    Halfclosed = "halfclosed",
}
