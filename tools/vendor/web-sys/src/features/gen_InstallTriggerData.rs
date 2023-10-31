#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = InstallTriggerData)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `InstallTriggerData` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `InstallTriggerData`*"]
    pub type InstallTriggerData;
}
impl InstallTriggerData {
    #[doc = "Construct a new `InstallTriggerData`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `InstallTriggerData`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `Hash` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `InstallTriggerData`*"]
    pub fn hash(&mut self, val: Option<&str>) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("Hash"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `IconURL` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `InstallTriggerData`*"]
    pub fn icon_url(&mut self, val: Option<&str>) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("IconURL"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `URL` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `InstallTriggerData`*"]
    pub fn url(&mut self, val: &str) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("URL"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for InstallTriggerData {
    fn default() -> Self {
        Self::new()
    }
}
