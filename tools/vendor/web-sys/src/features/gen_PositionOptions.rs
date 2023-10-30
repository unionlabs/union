#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = PositionOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `PositionOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PositionOptions`*"]
    pub type PositionOptions;
}
impl PositionOptions {
    #[doc = "Construct a new `PositionOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PositionOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `enableHighAccuracy` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PositionOptions`*"]
    pub fn enable_high_accuracy(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("enableHighAccuracy"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `maximumAge` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PositionOptions`*"]
    pub fn maximum_age(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("maximumAge"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `timeout` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `PositionOptions`*"]
    pub fn timeout(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("timeout"),
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
impl Default for PositionOptions {
    fn default() -> Self {
        Self::new()
    }
}
