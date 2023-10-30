#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = ConstantSourceOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `ConstantSourceOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ConstantSourceOptions`*"]
    pub type ConstantSourceOptions;
}
impl ConstantSourceOptions {
    #[doc = "Construct a new `ConstantSourceOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ConstantSourceOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `offset` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ConstantSourceOptions`*"]
    pub fn offset(&mut self, val: f32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("offset"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for ConstantSourceOptions {
    fn default() -> Self {
        Self::new()
    }
}
