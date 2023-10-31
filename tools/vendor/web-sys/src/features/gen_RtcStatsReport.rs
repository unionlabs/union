#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = RTCStatsReport , typescript_type = "RTCStatsReport")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `RtcStatsReport` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RTCStatsReport)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcStatsReport`*"]
    pub type RtcStatsReport;
}
