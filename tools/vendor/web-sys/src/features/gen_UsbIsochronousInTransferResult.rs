#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = USBIsochronousInTransferResult , typescript_type = "USBIsochronousInTransferResult")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `UsbIsochronousInTransferResult` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/USBIsochronousInTransferResult)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `UsbIsochronousInTransferResult`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub type UsbIsochronousInTransferResult;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , getter , js_class = "USBIsochronousInTransferResult" , js_name = data)]
    #[doc = "Getter for the `data` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/USBIsochronousInTransferResult/data)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `UsbIsochronousInTransferResult`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn data(this: &UsbIsochronousInTransferResult) -> Option<::js_sys::DataView>;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , getter , js_class = "USBIsochronousInTransferResult" , js_name = packets)]
    #[doc = "Getter for the `packets` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/USBIsochronousInTransferResult/packets)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `UsbIsochronousInTransferResult`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn packets(this: &UsbIsochronousInTransferResult) -> ::js_sys::Array;
    #[cfg(web_sys_unstable_apis)]
    #[wasm_bindgen(catch, constructor, js_class = "USBIsochronousInTransferResult")]
    #[doc = "The `new UsbIsochronousInTransferResult(..)` constructor, creating a new instance of `UsbIsochronousInTransferResult`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/USBIsochronousInTransferResult/USBIsochronousInTransferResult)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `UsbIsochronousInTransferResult`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new(
        packets: &::wasm_bindgen::JsValue,
    ) -> Result<UsbIsochronousInTransferResult, JsValue>;
    #[cfg(web_sys_unstable_apis)]
    #[wasm_bindgen(catch, constructor, js_class = "USBIsochronousInTransferResult")]
    #[doc = "The `new UsbIsochronousInTransferResult(..)` constructor, creating a new instance of `UsbIsochronousInTransferResult`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/USBIsochronousInTransferResult/USBIsochronousInTransferResult)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `UsbIsochronousInTransferResult`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new_with_data(
        packets: &::wasm_bindgen::JsValue,
        data: Option<&::js_sys::DataView>,
    ) -> Result<UsbIsochronousInTransferResult, JsValue>;
}
