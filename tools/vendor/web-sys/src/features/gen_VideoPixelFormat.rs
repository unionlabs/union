#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
#[doc = "The `VideoPixelFormat` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `VideoPixelFormat`*"]
#[doc = ""]
#[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
#[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoPixelFormat {
    I420 = "I420",
    I420a = "I420A",
    I422 = "I422",
    I444 = "I444",
    Nv12 = "NV12",
    Rgba = "RGBA",
    Rgbx = "RGBX",
    Bgra = "BGRA",
    Bgrx = "BGRX",
}
