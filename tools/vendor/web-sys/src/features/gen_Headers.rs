#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = Headers , typescript_type = "Headers")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `Headers` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub type Headers;
    #[wasm_bindgen(catch, constructor, js_class = "Headers")]
    #[doc = "The `new Headers(..)` constructor, creating a new instance of `Headers`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers/Headers)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub fn new() -> Result<Headers, JsValue>;
    #[wasm_bindgen(catch, constructor, js_class = "Headers")]
    #[doc = "The `new Headers(..)` constructor, creating a new instance of `Headers`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers/Headers)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub fn new_with_headers(init: &Headers) -> Result<Headers, JsValue>;
    #[wasm_bindgen(catch, constructor, js_class = "Headers")]
    #[doc = "The `new Headers(..)` constructor, creating a new instance of `Headers`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers/Headers)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub fn new_with_str_sequence_sequence(
        init: &::wasm_bindgen::JsValue,
    ) -> Result<Headers, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = append)]
    #[doc = "The `append()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers/append)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub fn append(this: &Headers, name: &str, value: &str) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = delete)]
    #[doc = "The `delete()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers/delete)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub fn delete(this: &Headers, name: &str) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = get)]
    #[doc = "The `get()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers/get)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub fn get(this: &Headers, name: &str) -> Result<Option<String>, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = has)]
    #[doc = "The `has()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers/has)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub fn has(this: &Headers, name: &str) -> Result<bool, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "Headers" , js_name = set)]
    #[doc = "The `set()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Headers/set)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Headers`*"]
    pub fn set(this: &Headers, name: &str, value: &str) -> Result<(), JsValue>;
}
