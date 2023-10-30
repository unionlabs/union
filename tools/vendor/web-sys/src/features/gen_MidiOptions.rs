#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = MIDIOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `MidiOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MidiOptions`*"]
    pub type MidiOptions;
}
impl MidiOptions {
    #[doc = "Construct a new `MidiOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MidiOptions`*"]
    pub fn new() -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret
    }
    #[doc = "Change the `software` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MidiOptions`*"]
    pub fn software(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("software"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `sysex` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MidiOptions`*"]
    pub fn sysex(&mut self, val: bool) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("sysex"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
}
impl Default for MidiOptions {
    fn default() -> Self {
        Self::new()
    }
}
