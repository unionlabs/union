#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = MIDIInputMap , typescript_type = "MIDIInputMap")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `MidiInputMap` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/MIDIInputMap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MidiInputMap`*"]
    pub type MidiInputMap;
}
