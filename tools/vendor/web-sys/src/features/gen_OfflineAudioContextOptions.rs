#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = OfflineAudioContextOptions)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `OfflineAudioContextOptions` dictionary."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `OfflineAudioContextOptions`*"]
    pub type OfflineAudioContextOptions;
}
impl OfflineAudioContextOptions {
    #[doc = "Construct a new `OfflineAudioContextOptions`."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `OfflineAudioContextOptions`*"]
    pub fn new(length: u32, sample_rate: f32) -> Self {
        #[allow(unused_mut)]
        let mut ret: Self = ::wasm_bindgen::JsCast::unchecked_into(::js_sys::Object::new());
        ret.length(length);
        ret.sample_rate(sample_rate);
        ret
    }
    #[doc = "Change the `length` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `OfflineAudioContextOptions`*"]
    pub fn length(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r =
            ::js_sys::Reflect::set(self.as_ref(), &JsValue::from("length"), &JsValue::from(val));
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `numberOfChannels` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `OfflineAudioContextOptions`*"]
    pub fn number_of_channels(&mut self, val: u32) -> &mut Self {
        use wasm_bindgen::JsValue;
        let r = ::js_sys::Reflect::set(
            self.as_ref(),
            &JsValue::from("numberOfChannels"),
            &JsValue::from(val),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;
        self
    }
    #[doc = "Change the `sampleRate` field of this object."]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `OfflineAudioContextOptions`*"]
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
