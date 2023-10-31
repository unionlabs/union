#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = CSSPseudoElement , typescript_type = "CSSPseudoElement")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `CssPseudoElement` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CSSPseudoElement)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CssPseudoElement`*"]
    pub type CssPseudoElement;
    # [wasm_bindgen (structural , method , getter , js_class = "CSSPseudoElement" , js_name = type)]
    #[doc = "Getter for the `type` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CSSPseudoElement/type)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CssPseudoElement`*"]
    pub fn type_(this: &CssPseudoElement) -> String;
    #[cfg(feature = "Element")]
    # [wasm_bindgen (structural , method , getter , js_class = "CSSPseudoElement" , js_name = parentElement)]
    #[doc = "Getter for the `parentElement` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/CSSPseudoElement/parentElement)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CssPseudoElement`, `Element`*"]
    pub fn parent_element(this: &CssPseudoElement) -> Element;
}
