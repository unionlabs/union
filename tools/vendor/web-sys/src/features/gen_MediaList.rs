#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = MediaList , typescript_type = "MediaList")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `MediaList` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaList)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaList`*"]
    pub type MediaList;
    # [wasm_bindgen (structural , method , getter , js_class = "MediaList" , js_name = mediaText)]
    #[doc = "Getter for the `mediaText` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/mediaText)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaList`*"]
    pub fn media_text(this: &MediaList) -> String;
    # [wasm_bindgen (structural , method , setter , js_class = "MediaList" , js_name = mediaText)]
    #[doc = "Setter for the `mediaText` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/mediaText)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaList`*"]
    pub fn set_media_text(this: &MediaList, value: &str);
    # [wasm_bindgen (structural , method , getter , js_class = "MediaList" , js_name = length)]
    #[doc = "Getter for the `length` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/length)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaList`*"]
    pub fn length(this: &MediaList) -> u32;
    # [wasm_bindgen (catch , method , structural , js_class = "MediaList" , js_name = appendMedium)]
    #[doc = "The `appendMedium()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/appendMedium)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaList`*"]
    pub fn append_medium(this: &MediaList, new_medium: &str) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "MediaList" , js_name = deleteMedium)]
    #[doc = "The `deleteMedium()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/deleteMedium)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaList`*"]
    pub fn delete_medium(this: &MediaList, old_medium: &str) -> Result<(), JsValue>;
    # [wasm_bindgen (method , structural , js_class = "MediaList" , js_name = item)]
    #[doc = "The `item()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MediaList/item)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaList`*"]
    pub fn item(this: &MediaList, index: u32) -> Option<String>;
    #[wasm_bindgen(method, structural, js_class = "MediaList", indexing_getter)]
    #[doc = "Indexing getter."]
    #[doc = ""]
    #[doc = ""]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaList`*"]
    pub fn get(this: &MediaList, index: u32) -> Option<String>;
}
