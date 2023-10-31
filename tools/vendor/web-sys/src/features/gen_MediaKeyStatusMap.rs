#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = MediaKeyStatusMap , typescript_type = "MediaKeyStatusMap")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `MediaKeyStatusMap` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaKeyStatusMap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaKeyStatusMap`*"]
    pub type MediaKeyStatusMap;
    # [wasm_bindgen (structural , method , getter , js_class = "MediaKeyStatusMap" , js_name = size)]
    #[doc = "Getter for the `size` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaKeyStatusMap/size)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaKeyStatusMap`*"]
    pub fn size(this: &MediaKeyStatusMap) -> u32;
    # [wasm_bindgen (catch , method , structural , js_class = "MediaKeyStatusMap" , js_name = get)]
    #[doc = "The `get()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaKeyStatusMap/get)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaKeyStatusMap`*"]
    pub fn get_with_buffer_source(
        this: &MediaKeyStatusMap,
        key_id: &::js_sys::Object,
    ) -> Result<::wasm_bindgen::JsValue, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "MediaKeyStatusMap" , js_name = get)]
    #[doc = "The `get()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaKeyStatusMap/get)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaKeyStatusMap`*"]
    pub fn get_with_u8_array(
        this: &MediaKeyStatusMap,
        key_id: &mut [u8],
    ) -> Result<::wasm_bindgen::JsValue, JsValue>;
    # [wasm_bindgen (method , structural , js_class = "MediaKeyStatusMap" , js_name = has)]
    #[doc = "The `has()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaKeyStatusMap/has)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaKeyStatusMap`*"]
    pub fn has_with_buffer_source(this: &MediaKeyStatusMap, key_id: &::js_sys::Object) -> bool;
    # [wasm_bindgen (method , structural , js_class = "MediaKeyStatusMap" , js_name = has)]
    #[doc = "The `has()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaKeyStatusMap/has)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaKeyStatusMap`*"]
    pub fn has_with_u8_array(this: &MediaKeyStatusMap, key_id: &mut [u8]) -> bool;
}
