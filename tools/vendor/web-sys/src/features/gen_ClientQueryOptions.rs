#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = ClientQueryOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `ClientQueryOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ClientQueryOptions`*"]
    pub type ClientQueryOptions;
}
impl ClientQueryOptions {
    #[doc = "Construct a new `ClientQueryOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ClientQueryOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `includeUncontrolled` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ClientQueryOptions`*"]
    pub fn include_uncontrolled(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("includeUncontrolled"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[cfg(feature = "ClientType")]
    #[doc = "Change the `type` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ClientQueryOptions`, `ClientType`*"]
    pub fn type_(&mut self, val: ClientType) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("type"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for ClientQueryOptions {
    fn default() -> Self {
        Self::new()
    }
}
