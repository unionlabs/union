#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = LifecycleCallbacks)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `LifecycleCallbacks` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `LifecycleCallbacks`*"]
    pub type LifecycleCallbacks;
}
impl LifecycleCallbacks {
    #[doc = "Construct a new `LifecycleCallbacks`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `LifecycleCallbacks`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `adoptedCallback` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `LifecycleCallbacks`*"]
    pub fn adopted_callback(&mut self, val: &::js_sys::Function) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("adoptedCallback"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `attributeChangedCallback` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `LifecycleCallbacks`*"]
    pub fn attribute_changed_callback(&mut self, val: &::js_sys::Function) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("attributeChangedCallback"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `connectedCallback` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `LifecycleCallbacks`*"]
    pub fn connected_callback(&mut self, val: &::js_sys::Function) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("connectedCallback"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `disconnectedCallback` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `LifecycleCallbacks`*"]
    pub fn disconnected_callback(&mut self, val: &::js_sys::Function) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("disconnectedCallback"),
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
impl Default for LifecycleCallbacks {
    fn default() -> Self {
        Self::new()
    }
}
