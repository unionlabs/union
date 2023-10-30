#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = WorkerGlobalScope , extends = EventTarget , extends = :: js_sys :: Object , js_name = DedicatedWorkerGlobalScope , typescript_type = "DedicatedWorkerGlobalScope")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `DedicatedWorkerGlobalScope` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub type DedicatedWorkerGlobalScope;
    # [wasm_bindgen (structural , method , getter , js_class = "DedicatedWorkerGlobalScope" , js_name = name)]
    #[doc = "Getter for the `name` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/name)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub fn name(this: &DedicatedWorkerGlobalScope) -> String;
    # [wasm_bindgen (structural , method , getter , js_class = "DedicatedWorkerGlobalScope" , js_name = onmessage)]
    #[doc = "Getter for the `onmessage` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/onmessage)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub fn onmessage(this: &DedicatedWorkerGlobalScope) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "DedicatedWorkerGlobalScope" , js_name = onmessage)]
    #[doc = "Setter for the `onmessage` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/onmessage)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub fn set_onmessage(this: &DedicatedWorkerGlobalScope, value: Option<&::js_sys::Function>);
    # [wasm_bindgen (structural , method , getter , js_class = "DedicatedWorkerGlobalScope" , js_name = onmessageerror)]
    #[doc = "Getter for the `onmessageerror` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/onmessageerror)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub fn onmessageerror(this: &DedicatedWorkerGlobalScope) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "DedicatedWorkerGlobalScope" , js_name = onmessageerror)]
    #[doc = "Setter for the `onmessageerror` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/onmessageerror)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub fn set_onmessageerror(
        this: &DedicatedWorkerGlobalScope,
        value: Option<&::js_sys::Function>,
    );
    # [wasm_bindgen (method , structural , js_class = "DedicatedWorkerGlobalScope" , js_name = close)]
    #[doc = "The `close()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/close)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub fn close(this: &DedicatedWorkerGlobalScope);
    # [wasm_bindgen (catch , method , structural , js_class = "DedicatedWorkerGlobalScope" , js_name = postMessage)]
    #[doc = "The `postMessage()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/postMessage)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub fn post_message(
        this: &DedicatedWorkerGlobalScope,
        message: &::wasm_bindgen::JsValue,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "DedicatedWorkerGlobalScope" , js_name = postMessage)]
    #[doc = "The `postMessage()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/DedicatedWorkerGlobalScope/postMessage)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DedicatedWorkerGlobalScope`*"]
    pub fn post_message_with_transfer(
        this: &DedicatedWorkerGlobalScope,
        message: &::wasm_bindgen::JsValue,
        transfer: &::wasm_bindgen::JsValue,
    ) -> Result<(), JsValue>;
}
