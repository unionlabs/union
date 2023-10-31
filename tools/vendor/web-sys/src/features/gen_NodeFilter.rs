#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = NodeFilter)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `NodeFilter` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `NodeFilter`*"]
    pub type NodeFilter;
}
impl NodeFilter {
    #[doc = "Construct a new `NodeFilter`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `NodeFilter`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `acceptNode` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `NodeFilter`*"]
    pub fn accept_node(&mut self, val: &::js_sys::Function) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("acceptNode"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for NodeFilter {
    fn default() -> Self {
        Self::new()
    }
}
