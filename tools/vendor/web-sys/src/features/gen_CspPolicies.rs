#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = CSPPolicies)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `CspPolicies` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspPolicies`*"]
    pub type CspPolicies;
}
impl CspPolicies {
    #[doc = "Construct a new `CspPolicies`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspPolicies`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `csp-policies` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspPolicies`*"]
    pub fn csp_policies(&mut self, val: &::wasm_bindgen::JsValue) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("csp-policies"),
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
impl Default for CspPolicies {
    fn default() -> Self {
        Self::new()
    }
}
