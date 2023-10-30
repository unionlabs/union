#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
#[doc = "The `VideoTransferCharacteristics` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `VideoTransferCharacteristics`*"]
#[doc = ""]
#[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
#[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoTransferCharacteristics {
    Bt709 = "bt709",
    Smpte170m = "smpte170m",
    Iec6196621 = "iec61966-2-1",
}
