#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = RequestMediaKeySystemAccessNotification)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `RequestMediaKeySystemAccessNotification` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestMediaKeySystemAccessNotification`*"]
    pub type RequestMediaKeySystemAccessNotification;
}
impl RequestMediaKeySystemAccessNotification {
    #[cfg(feature = "MediaKeySystemStatus")]
    #[doc = "Construct a new `RequestMediaKeySystemAccessNotification`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaKeySystemStatus`, `RequestMediaKeySystemAccessNotification`*"]
    pub fn new(key_system: &str, status: MediaKeySystemStatus) -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret.key_system(key_system);
        ret.status(status);
        ret
    }
    #[doc = "Change the `keySystem` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestMediaKeySystemAccessNotification`*"]
    pub fn key_system(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("keySystem"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[cfg(feature = "MediaKeySystemStatus")]
    #[doc = "Change the `status` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaKeySystemStatus`, `RequestMediaKeySystemAccessNotification`*"]
    pub fn status(&mut self, val: MediaKeySystemStatus) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("status"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
