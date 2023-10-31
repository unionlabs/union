#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = NodeList , typescript_type = "NodeList")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `NodeList` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/NodeList)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `NodeList`*"]
    pub type NodeList;
    # [wasm_bindgen (structural , method , getter , js_class = "NodeList" , js_name = length)]
    #[doc = "Getter for the `length` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/NodeList/length)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `NodeList`*"]
    pub fn length(this: &NodeList) -> u32;
    #[cfg(feature = "Node")]
    # [wasm_bindgen (method , structural , js_class = "NodeList" , js_name = item)]
    #[doc = "The `item()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/NodeList/item)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Node`, `NodeList`*"]
    pub fn item(this: &NodeList, index: u32) -> Option<Node>;
    #[cfg(feature = "Node")]
    #[wasm_bindgen(method, structural, js_class = "NodeList", indexing_getter)]
    #[doc = "Indexing getter."]
    #[doc = ""]
    #[doc = ""]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Node`, `NodeList`*"]
    pub fn get(this: &NodeList, index: u32) -> Option<Node>;
}
