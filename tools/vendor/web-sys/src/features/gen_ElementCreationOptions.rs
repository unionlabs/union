#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = ElementCreationOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `ElementCreationOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ElementCreationOptions`*"]
    pub type ElementCreationOptions;
}
impl ElementCreationOptions {
    #[doc = "Construct a new `ElementCreationOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ElementCreationOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `is` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ElementCreationOptions`*"]
    pub fn is(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("is"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `pseudo` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ElementCreationOptions`*"]
    pub fn pseudo(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("pseudo"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for ElementCreationOptions {
    fn default() -> Self {
        Self::new()
    }
}
