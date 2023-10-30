#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = AudioParamMap , typescript_type = "AudioParamMap")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `AudioParamMap` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/AudioParamMap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `AudioParamMap`*"]
    pub type AudioParamMap;
}
