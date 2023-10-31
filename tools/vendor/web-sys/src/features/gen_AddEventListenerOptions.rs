#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = AddEventListenerOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `AddEventListenerOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AddEventListenerOptions`*"]
    pub type AddEventListenerOptions;
}
impl AddEventListenerOptions {
    #[doc = "Construct a new `AddEventListenerOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AddEventListenerOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `capture` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AddEventListenerOptions`*"]
    pub fn capture(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("capture"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `once` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AddEventListenerOptions`*"]
    pub fn once(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("once"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `passive` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AddEventListenerOptions`*"]
    pub fn passive(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("passive"),
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
impl Default for AddEventListenerOptions {
    fn default() -> Self {
        Self::new()
    }
}
