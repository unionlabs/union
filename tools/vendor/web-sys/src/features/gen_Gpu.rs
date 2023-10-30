#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = GPU , typescript_type = "GPU")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `Gpu` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GPU)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Gpu`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub type Gpu;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "GpuTextureFormat")]
    # [wasm_bindgen (method , structural , js_class = "GPU" , js_name = getPreferredCanvasFormat)]
    #[doc = "The `getPreferredCanvasFormat()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GPU/getPreferredCanvasFormat)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Gpu`, `GpuTextureFormat`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn get_preferred_canvas_format(this: &Gpu) -> GpuTextureFormat;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (method , structural , js_class = "GPU" , js_name = requestAdapter)]
    #[doc = "The `requestAdapter()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GPU/requestAdapter)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Gpu`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn request_adapter(this: &Gpu) -> ::js_sys::Promise;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "GpuRequestAdapterOptions")]
    # [wasm_bindgen (method , structural , js_class = "GPU" , js_name = requestAdapter)]
    #[doc = "The `requestAdapter()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GPU/requestAdapter)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Gpu`, `GpuRequestAdapterOptions`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn request_adapter_with_options(
        this: &Gpu,
        options: &GpuRequestAdapterOptions,
    ) -> ::js_sys::Promise;
}
