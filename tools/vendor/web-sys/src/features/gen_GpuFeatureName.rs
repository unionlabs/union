#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
#[doc = "The `GpuFeatureName` enum."]
#[doc = ""]
#[doc = "*This API requires the following crate features to be activated: `GpuFeatureName`*"]
#[doc = ""]
#[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
#[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuFeatureName {
    DepthClipControl = "depth-clip-control",
    Depth24unormStencil8 = "depth24unorm-stencil8",
    Depth32floatStencil8 = "depth32float-stencil8",
    TextureCompressionBc = "texture-compression-bc",
    TextureCompressionEtc2 = "texture-compression-etc2",
    TextureCompressionAstc = "texture-compression-astc",
    TimestampQuery = "timestamp-query",
    IndirectFirstInstance = "indirect-first-instance",
    ShaderF16 = "shader-f16",
    Bgra8unormStorage = "bgra8unorm-storage",
}
