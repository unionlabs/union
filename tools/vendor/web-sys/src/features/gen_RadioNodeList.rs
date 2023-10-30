#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = NodeList , extends = :: js_sys :: Object , js_name = RadioNodeList , typescript_type = "RadioNodeList")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `RadioNodeList` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RadioNodeList)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RadioNodeList`*"]
    pub type RadioNodeList;
    # [wasm_bindgen (structural , method , getter , js_class = "RadioNodeList" , js_name = value)]
    #[doc = "Getter for the `value` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RadioNodeList/value)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RadioNodeList`*"]
    pub fn value(this: &RadioNodeList) -> String;
    # [wasm_bindgen (structural , method , setter , js_class = "RadioNodeList" , js_name = value)]
    #[doc = "Setter for the `value` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RadioNodeList/value)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RadioNodeList`*"]
    pub fn set_value(this: &RadioNodeList, value: &str);
}
