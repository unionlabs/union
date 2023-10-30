#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = DecoderDoctorNotification)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `DecoderDoctorNotification` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`*"]
    pub type DecoderDoctorNotification;
}
impl DecoderDoctorNotification {
    #[cfg(feature = "DecoderDoctorNotificationType")]
    #[doc = "Construct a new `DecoderDoctorNotification`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`, `DecoderDoctorNotificationType`*"]
    pub fn new(
        decoder_doctor_report_id: &str,
        is_solved: bool,
        type_: DecoderDoctorNotificationType,
    ) -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret.decoder_doctor_report_id(decoder_doctor_report_id);
        ret.is_solved(is_solved);
        ret.type_(type_);
        ret
    }
    #[doc = "Change the `decodeIssue` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`*"]
    pub fn decode_issue(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("decodeIssue"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `decoderDoctorReportId` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`*"]
    pub fn decoder_doctor_report_id(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("decoderDoctorReportId"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `docURL` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`*"]
    pub fn doc_url(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("docURL"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `formats` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`*"]
    pub fn formats(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("formats"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `isSolved` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`*"]
    pub fn is_solved(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("isSolved"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `resourceURL` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`*"]
    pub fn resource_url(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("resourceURL"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[cfg(feature = "DecoderDoctorNotificationType")]
    #[doc = "Change the `type` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `DecoderDoctorNotification`, `DecoderDoctorNotificationType`*"]
    pub fn type_(&mut self, val: DecoderDoctorNotificationType) -> &mut Self {
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
