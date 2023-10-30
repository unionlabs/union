#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = WorkletOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `WorkletOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkletOptions`*"]
    pub type WorkletOptions;
}
impl WorkletOptions {
    #[doc = "Construct a new `WorkletOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkletOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[cfg(feature = "RequestCredentials")]
    #[doc = "Change the `credentials` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestCredentials`, `WorkletOptions`*"]
    pub fn credentials(&mut self, val: RequestCredentials) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("credentials"),
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
impl Default for WorkletOptions {
    fn default() -> Self {
        Self::new()
    }
}
