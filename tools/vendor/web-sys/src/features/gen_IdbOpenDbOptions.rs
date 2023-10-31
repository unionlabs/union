#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = IDBOpenDBOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `IdbOpenDbOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbOpenDbOptions`*"]
    pub type IdbOpenDbOptions;
}
impl IdbOpenDbOptions {
    #[doc = "Construct a new `IdbOpenDbOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbOpenDbOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[cfg(feature = "StorageType")]
    #[doc = "Change the `storage` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbOpenDbOptions`, `StorageType`*"]
    pub fn storage(&mut self, val: StorageType) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("storage"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `version` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbOpenDbOptions`*"]
    pub fn version(&mut self, val: f64) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("version"),
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
impl Default for IdbOpenDbOptions {
    fn default() -> Self {
        Self::new()
    }
}
