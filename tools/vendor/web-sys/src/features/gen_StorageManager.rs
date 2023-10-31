#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = StorageManager , typescript_type = "StorageManager")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `StorageManager` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/StorageManager)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `StorageManager`*"]
    pub type StorageManager;
    # [wasm_bindgen (catch , method , structural , js_class = "StorageManager" , js_name = estimate)]
    #[doc = "The `estimate()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/StorageManager/estimate)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `StorageManager`*"]
    pub fn estimate(this: &StorageManager) -> Result<::js_sys::Promise, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "StorageManager" , js_name = persist)]
    #[doc = "The `persist()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/StorageManager/persist)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `StorageManager`*"]
    pub fn persist(this: &StorageManager) -> Result<::js_sys::Promise, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "StorageManager" , js_name = persisted)]
    #[doc = "The `persisted()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/StorageManager/persisted)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `StorageManager`*"]
    pub fn persisted(this: &StorageManager) -> Result<::js_sys::Promise, JsValue>;
}
