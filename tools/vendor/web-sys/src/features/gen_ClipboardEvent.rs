#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = Event , extends = :: js_sys :: Object , js_name = ClipboardEvent , typescript_type = "ClipboardEvent")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `ClipboardEvent` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ClipboardEvent`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub type ClipboardEvent;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "DataTransfer")]
    # [wasm_bindgen (structural , method , getter , js_class = "ClipboardEvent" , js_name = clipboardData)]
    #[doc = "Getter for the `clipboardData` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent/clipboardData)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ClipboardEvent`, `DataTransfer`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn clipboard_data(this: &ClipboardEvent) -> Option<DataTransfer>;
    #[cfg(web_sys_unstable_apis)]
    #[wasm_bindgen(catch, constructor, js_class = "ClipboardEvent")]
    #[doc = "The `new ClipboardEvent(..)` constructor, creating a new instance of `ClipboardEvent`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent/ClipboardEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ClipboardEvent`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new(type_: &str) -> Result<ClipboardEvent, JsValue>;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "ClipboardEventInit")]
    #[wasm_bindgen(catch, constructor, js_class = "ClipboardEvent")]
    #[doc = "The `new ClipboardEvent(..)` constructor, creating a new instance of `ClipboardEvent`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ClipboardEvent/ClipboardEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ClipboardEvent`, `ClipboardEventInit`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new_with_event_init_dict(
        type_: &str,
        event_init_dict: &ClipboardEventInit,
    ) -> Result<ClipboardEvent, JsValue>;
}
