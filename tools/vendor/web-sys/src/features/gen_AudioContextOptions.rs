#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = AudioContextOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `AudioContextOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AudioContextOptions`*"]
    pub type AudioContextOptions;
}
impl AudioContextOptions {
    #[doc = "Construct a new `AudioContextOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AudioContextOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `sampleRate` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AudioContextOptions`*"]
    pub fn sample_rate(&mut self, val: f32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("sampleRate"),
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
impl Default for AudioContextOptions {
    fn default() -> Self {
        Self::new()
    }
}
