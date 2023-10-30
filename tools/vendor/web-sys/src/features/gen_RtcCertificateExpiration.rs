#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = RTCCertificateExpiration)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `RtcCertificateExpiration` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcCertificateExpiration`*"]
    pub type RtcCertificateExpiration;
}
impl RtcCertificateExpiration {
    #[doc = "Construct a new `RtcCertificateExpiration`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcCertificateExpiration`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `expires` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcCertificateExpiration`*"]
    pub fn expires(&mut self, val: f64) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("expires"),
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
impl Default for RtcCertificateExpiration {
    fn default() -> Self {
        Self::new()
    }
}
