#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = IDBObjectStoreParameters)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `IdbObjectStoreParameters` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbObjectStoreParameters`*"]
    pub type IdbObjectStoreParameters;
}
impl IdbObjectStoreParameters {
    #[doc = "Construct a new `IdbObjectStoreParameters`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbObjectStoreParameters`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `autoIncrement` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbObjectStoreParameters`*"]
    pub fn auto_increment(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("autoIncrement"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `keyPath` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbObjectStoreParameters`*"]
    pub fn key_path(&mut self, val: Option<&::wasm_bindgen::JsValue>) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("keyPath"),
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
impl Default for IdbObjectStoreParameters {
    fn default() -> Self {
        Self::new()
    }
}
