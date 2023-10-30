#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = RTCRtpReceiver , typescript_type = "RTCRtpReceiver")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `RtcRtpReceiver` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RTCRtpReceiver)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcRtpReceiver`*"]
    pub type RtcRtpReceiver;
    #[cfg(feature = "MediaStreamTrack")]
    # [wasm_bindgen (structural , method , getter , js_class = "RTCRtpReceiver" , js_name = track)]
    #[doc = "Getter for the `track` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RTCRtpReceiver/track)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `MediaStreamTrack`, `RtcRtpReceiver`*"]
    pub fn track(this: &RtcRtpReceiver) -> MediaStreamTrack;
    # [wasm_bindgen (method , structural , js_class = "RTCRtpReceiver" , js_name = getContributingSources)]
    #[doc = "The `getContributingSources()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RTCRtpReceiver/getContributingSources)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcRtpReceiver`*"]
    pub fn get_contributing_sources(this: &RtcRtpReceiver) -> ::js_sys::Array;
    # [wasm_bindgen (method , structural , js_class = "RTCRtpReceiver" , js_name = getStats)]
    #[doc = "The `getStats()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RTCRtpReceiver/getStats)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcRtpReceiver`*"]
    pub fn get_stats(this: &RtcRtpReceiver) -> ::js_sys::Promise;
    # [wasm_bindgen (method , structural , js_class = "RTCRtpReceiver" , js_name = getSynchronizationSources)]
    #[doc = "The `getSynchronizationSources()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/RTCRtpReceiver/getSynchronizationSources)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RtcRtpReceiver`*"]
    pub fn get_synchronization_sources(this: &RtcRtpReceiver) -> ::js_sys::Array;
}
