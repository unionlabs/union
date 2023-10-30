#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = MediaStream , extends = EventTarget , extends = :: js_sys :: Object , js_name = LocalMediaStream , typescript_type = "LocalMediaStream")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `LocalMediaStream` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/LocalMediaStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `LocalMediaStream`*"]
    pub type LocalMediaStream;
    # [wasm_bindgen (method , structural , js_class = "LocalMediaStream" , js_name = stop)]
    #[doc = "The `stop()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/LocalMediaStream/stop)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `LocalMediaStream`*"]
    pub fn stop(this: &LocalMediaStream);
}
