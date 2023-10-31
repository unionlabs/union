#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = RTCSessionDescriptionInit)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `RtcSessionDescriptionInit` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcSessionDescriptionInit`*"]
    pub type RtcSessionDescriptionInit;
}
impl RtcSessionDescriptionInit {
    #[cfg(feature = "RtcSdpType")]
    #[doc = "Construct a new `RtcSessionDescriptionInit`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcSdpType`, `RtcSessionDescriptionInit`*"]
    pub fn new(type_: RtcSdpType) -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret.type_(type_);
        ret
    }
    #[doc = "Change the `sdp` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcSessionDescriptionInit`*"]
    pub fn sdp(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("sdp"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[cfg(feature = "RtcSdpType")]
    #[doc = "Change the `type` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcSdpType`, `RtcSessionDescriptionInit`*"]
    pub fn type_(&mut self, val: RtcSdpType) -> &mut Self {
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
