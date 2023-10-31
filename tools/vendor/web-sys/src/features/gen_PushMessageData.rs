#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = PushMessageData , typescript_type = "PushMessageData")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `PushMessageData` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/PushMessageData)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PushMessageData`*"]
    pub type PushMessageData;
    # [wasm_bindgen (catch , method , structural , js_class = "PushMessageData" , js_name = arrayBuffer)]
    #[doc = "The `arrayBuffer()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/PushMessageData/arrayBuffer)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PushMessageData`*"]
    pub fn array_buffer(this: &PushMessageData) -> Result<::js_sys::ArrayBuffer, JsValue>;
    #[cfg(feature = "Blob")]
    # [wasm_bindgen (catch , method , structural , js_class = "PushMessageData" , js_name = blob)]
    #[doc = "The `blob()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/PushMessageData/blob)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Blob`, `PushMessageData`*"]
    pub fn blob(this: &PushMessageData) -> Result<Blob, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "PushMessageData" , js_name = json)]
    #[doc = "The `json()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/PushMessageData/json)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PushMessageData`*"]
    pub fn json(this: &PushMessageData) -> Result<::wasm_bindgen::JsValue, JsValue>;
    # [wasm_bindgen (method , structural , js_class = "PushMessageData" , js_name = text)]
    #[doc = "The `text()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/PushMessageData/text)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PushMessageData`*"]
    pub fn text(this: &PushMessageData) -> String;
}
