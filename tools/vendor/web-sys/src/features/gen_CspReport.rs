#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = CSPReport)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `CspReport` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReport`*"]
    pub type CspReport;
}
impl CspReport {
    #[doc = "Construct a new `CspReport`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReport`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[cfg(feature = "CspReportProperties")]
    #[doc = "Change the `csp-report` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CspReport`, `CspReportProperties`*"]
    pub fn csp_report(&mut self, val: &CspReportProperties) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("csp-report"),
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
impl Default for CspReport {
    fn default() -> Self {
        Self::new()
    }
}
