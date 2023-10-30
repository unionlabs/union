#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = KeyIdsInitData)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `KeyIdsInitData` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `KeyIdsInitData`*"]
    pub type KeyIdsInitData;
}
impl KeyIdsInitData {
    #[doc = "Construct a new `KeyIdsInitData`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `KeyIdsInitData`*"]
    pub fn new(kids: &::wasm_bindgen::JsValue) -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret.kids(kids);
        ret
    }
    #[doc = "Change the `kids` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `KeyIdsInitData`*"]
    pub fn kids(&mut self, val: &::wasm_bindgen::JsValue) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("kids"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
