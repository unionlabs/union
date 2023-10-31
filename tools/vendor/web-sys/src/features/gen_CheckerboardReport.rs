#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = CheckerboardReport)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `CheckerboardReport` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CheckerboardReport`*"]
    pub type CheckerboardReport;
}
impl CheckerboardReport {
    #[doc = "Construct a new `CheckerboardReport`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CheckerboardReport`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `log` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CheckerboardReport`*"]
    pub fn log(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("log"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[cfg(feature = "CheckerboardReason")]
    #[doc = "Change the `reason` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CheckerboardReason`, `CheckerboardReport`*"]
    pub fn reason(&mut self, val: CheckerboardReason) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("reason"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `severity` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CheckerboardReport`*"]
    pub fn severity(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("severity"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `timestamp` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CheckerboardReport`*"]
    pub fn timestamp(&mut self, val: f64) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("timestamp"),
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
impl Default for CheckerboardReport {
    fn default() -> Self {
        Self::new()
    }
}
