#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[doc = "The `WebGlPowerPreference` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `WebGlPowerPreference`*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebGlPowerPreference {
    Default = "default",
    LowPower = "low-power",
    HighPerformance = "high-performance",
}
