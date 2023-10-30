#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
#[doc = "The `GpuBlendFactor` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `GpuBlendFactor`*"]
#[doc = ""]
#[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
#[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBlendFactor {
    Zero = "zero",
    One = "one",
    Src = "src",
    OneMinusSrc = "one-minus-src",
    SrcAlpha = "src-alpha",
    OneMinusSrcAlpha = "one-minus-src-alpha",
    Dst = "dst",
    OneMinusDst = "one-minus-dst",
    DstAlpha = "dst-alpha",
    OneMinusDstAlpha = "one-minus-dst-alpha",
    SrcAlphaSaturated = "src-alpha-saturated",
    Constant = "constant",
    OneMinusConstant = "one-minus-constant",
}
