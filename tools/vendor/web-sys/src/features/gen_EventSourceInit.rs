#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = EventSourceInit)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `EventSourceInit` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `EventSourceInit`*"]
    pub type EventSourceInit;
}
impl EventSourceInit {
    #[doc = "Construct a new `EventSourceInit`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `EventSourceInit`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `withCredentials` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `EventSourceInit`*"]
    pub fn with_credentials(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("withCredentials"),
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
impl Default for EventSourceInit {
    fn default() -> Self {
        Self::new()
    }
}
