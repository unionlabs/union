#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = TransformStream , typescript_type = "TransformStream")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `TransformStream` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `TransformStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub type TransformStream;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "ReadableStream")]
    # [wasm_bindgen (structural , method , getter , js_class = "TransformStream" , js_name = readable)]
    #[doc = "Getter for the `readable` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream/readable)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`, `TransformStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn readable(this: &TransformStream) -> ReadableStream;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "WritableStream")]
    # [wasm_bindgen (structural , method , getter , js_class = "TransformStream" , js_name = writable)]
    #[doc = "Getter for the `writable` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream/writable)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `TransformStream`, `WritableStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn writable(this: &TransformStream) -> WritableStream;
    #[cfg(web_sys_unstable_apis)]
    #[wasm_bindgen(catch, constructor, js_class = "TransformStream")]
    #[doc = "The `new TransformStream(..)` constructor, creating a new instance of `TransformStream`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream/TransformStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `TransformStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new() -> Result<TransformStream, JsValue>;
    #[cfg(web_sys_unstable_apis)]
    #[wasm_bindgen(catch, constructor, js_class = "TransformStream")]
    #[doc = "The `new TransformStream(..)` constructor, creating a new instance of `TransformStream`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream/TransformStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `TransformStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new_with_transformer(transformer: &::js_sys::Object)
        -> Result<TransformStream, JsValue>;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "QueuingStrategy")]
    #[wasm_bindgen(catch, constructor, js_class = "TransformStream")]
    #[doc = "The `new TransformStream(..)` constructor, creating a new instance of `TransformStream`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream/TransformStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `QueuingStrategy`, `TransformStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new_with_transformer_and_writable_strategy(
        transformer: &::js_sys::Object,
        writable_strategy: &QueuingStrategy,
    ) -> Result<TransformStream, JsValue>;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "QueuingStrategy")]
    #[wasm_bindgen(catch, constructor, js_class = "TransformStream")]
    #[doc = "The `new TransformStream(..)` constructor, creating a new instance of `TransformStream`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/TransformStream/TransformStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `QueuingStrategy`, `TransformStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new_with_transformer_and_writable_strategy_and_readable_strategy(
        transformer: &::js_sys::Object,
        writable_strategy: &QueuingStrategy,
        readable_strategy: &QueuingStrategy,
    ) -> Result<TransformStream, JsValue>;
}
