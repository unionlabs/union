#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = GetRootNodeOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `GetRootNodeOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GetRootNodeOptions`*"]
    pub type GetRootNodeOptions;
}
impl GetRootNodeOptions {
    #[doc = "Construct a new `GetRootNodeOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GetRootNodeOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `composed` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GetRootNodeOptions`*"]
    pub fn composed(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("composed"),
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
impl Default for GetRootNodeOptions {
    fn default() -> Self {
        Self::new()
    }
}
