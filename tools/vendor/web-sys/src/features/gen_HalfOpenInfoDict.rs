#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = HalfOpenInfoDict)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `HalfOpenInfoDict` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HalfOpenInfoDict`*"]
    pub type HalfOpenInfoDict;
}
impl HalfOpenInfoDict {
    #[doc = "Construct a new `HalfOpenInfoDict`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HalfOpenInfoDict`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `speculative` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HalfOpenInfoDict`*"]
    pub fn speculative(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("speculative"),
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
impl Default for HalfOpenInfoDict {
    fn default() -> Self {
        Self::new()
    }
}
