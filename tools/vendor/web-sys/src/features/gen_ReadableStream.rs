#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = ReadableStream , typescript_type = "ReadableStream")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `ReadableStream` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`*"]
    pub type ReadableStream;
    # [wasm_bindgen (structural , method , getter , js_class = "ReadableStream" , js_name = locked)]
    #[doc = "Getter for the `locked` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/locked)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`*"]
    pub fn locked(this: &ReadableStream) -> bool;
    #[cfg(web_sys_unstable_apis)]
    #[wasm_bindgen(catch, constructor, js_class = "ReadableStream")]
    #[doc = "The `new ReadableStream(..)` constructor, creating a new instance of `ReadableStream`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/ReadableStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new() -> Result<ReadableStream, JsValue>;
    #[cfg(web_sys_unstable_apis)]
    #[wasm_bindgen(catch, constructor, js_class = "ReadableStream")]
    #[doc = "The `new ReadableStream(..)` constructor, creating a new instance of `ReadableStream`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/ReadableStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new_with_underlying_source(
        underlying_source: &::js_sys::Object,
    ) -> Result<ReadableStream, JsValue>;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "QueuingStrategy")]
    #[wasm_bindgen(catch, constructor, js_class = "ReadableStream")]
    #[doc = "The `new ReadableStream(..)` constructor, creating a new instance of `ReadableStream`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/ReadableStream)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `QueuingStrategy`, `ReadableStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn new_with_underlying_source_and_strategy(
        underlying_source: &::js_sys::Object,
        strategy: &QueuingStrategy,
    ) -> Result<ReadableStream, JsValue>;
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = cancel)]
    #[doc = "The `cancel()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/cancel)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`*"]
    pub fn cancel(this: &ReadableStream) -> ::js_sys::Promise;
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = cancel)]
    #[doc = "The `cancel()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/cancel)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`*"]
    pub fn cancel_with_reason(
        this: &ReadableStream,
        reason: &::wasm_bindgen::JsValue,
    ) -> ::js_sys::Promise;
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = getReader)]
    #[doc = "The `getReader()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/getReader)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`*"]
    pub fn get_reader(this: &ReadableStream) -> ::js_sys::Object;
    #[cfg(feature = "ReadableStreamGetReaderOptions")]
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = getReader)]
    #[doc = "The `getReader()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/getReader)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`, `ReadableStreamGetReaderOptions`*"]
    pub fn get_reader_with_options(
        this: &ReadableStream,
        options: &ReadableStreamGetReaderOptions,
    ) -> ::js_sys::Object;
    #[cfg(feature = "ReadableWritablePair")]
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = pipeThrough)]
    #[doc = "The `pipeThrough()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeThrough)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`, `ReadableWritablePair`*"]
    pub fn pipe_through(this: &ReadableStream, transform: &ReadableWritablePair) -> ReadableStream;
    #[cfg(all(feature = "ReadableWritablePair", feature = "StreamPipeOptions",))]
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = pipeThrough)]
    #[doc = "The `pipeThrough()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeThrough)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`, `ReadableWritablePair`, `StreamPipeOptions`*"]
    pub fn pipe_through_with_options(
        this: &ReadableStream,
        transform: &ReadableWritablePair,
        options: &StreamPipeOptions,
    ) -> ReadableStream;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "WritableStream")]
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = pipeTo)]
    #[doc = "The `pipeTo()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeTo)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`, `WritableStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn pipe_to(this: &ReadableStream, destination: &WritableStream) -> ::js_sys::Promise;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(all(feature = "StreamPipeOptions", feature = "WritableStream",))]
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = pipeTo)]
    #[doc = "The `pipeTo()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/pipeTo)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`, `StreamPipeOptions`, `WritableStream`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn pipe_to_with_options(
        this: &ReadableStream,
        destination: &WritableStream,
        options: &StreamPipeOptions,
    ) -> ::js_sys::Promise;
    # [wasm_bindgen (method , structural , js_class = "ReadableStream" , js_name = tee)]
    #[doc = "The `tee()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream/tee)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ReadableStream`*"]
    pub fn tee(this: &ReadableStream) -> ::js_sys::Array;
}
