#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = CSPReportProperties)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `CspReportProperties` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub type CspReportProperties;
}
impl CspReportProperties {
    #[doc = "Construct a new `CspReportProperties`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `blocked-uri` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn blocked_uri(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("blocked-uri"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `column-number` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn column_number(&mut self, val: i32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("column-number"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `document-uri` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn document_uri(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("document-uri"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `line-number` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn line_number(&mut self, val: i32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("line-number"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `original-policy` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn original_policy(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("original-policy"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `referrer` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn referrer(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("referrer"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `script-sample` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn script_sample(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("script-sample"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `source-file` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn source_file(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("source-file"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `violated-directive` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReportProperties`*"]
    pub fn violated_directive(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("violated-directive"),
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
impl Default for CspReportProperties {
    fn default() -> Self {
        Self::new()
    }
}
