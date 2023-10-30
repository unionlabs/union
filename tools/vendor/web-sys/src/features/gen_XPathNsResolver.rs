#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = XPathNSResolver)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `XPathNsResolver` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `XPathNsResolver`*"]
    pub type XPathNsResolver;
}
impl XPathNsResolver {
    #[doc = "Construct a new `XPathNsResolver`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `XPathNsResolver`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `lookupNamespaceURI` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `XPathNsResolver`*"]
    pub fn lookup_namespace_uri(&mut self, val: &::js_sys::Function) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("lookupNamespaceURI"),
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
impl Default for XPathNsResolver {
    fn default() -> Self {
        Self::new()
    }
}
