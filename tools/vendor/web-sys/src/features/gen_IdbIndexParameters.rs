#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = IDBIndexParameters)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `IdbIndexParameters` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndexParameters`*"]
    pub type IdbIndexParameters;
}
impl IdbIndexParameters {
    #[doc = "Construct a new `IdbIndexParameters`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndexParameters`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `locale` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndexParameters`*"]
    pub fn locale(&mut self, val: Option<&str>) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("locale"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `multiEntry` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndexParameters`*"]
    pub fn multi_entry(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("multiEntry"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `unique` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbIndexParameters`*"]
    pub fn unique(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("unique"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for IdbIndexParameters {
    fn default() -> Self {
        Self::new()
    }
}
