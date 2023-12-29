use js_sys::Object;
use wasm_bindgen::prelude::*;

// WebAssembly.Global
#[wasm_bindgen]
extern "C" {
    /// The `WebAssembly.Global()` constructor creates a new `Global` object
    /// of the given type and value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Global)
    #[wasm_bindgen(js_namespace = WebAssembly, extends = Object, typescript_type = "WebAssembly.Global")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type Global;

    /// The `WebAssembly.Global()` constructor creates a new `Global` object
    /// of the given type and value.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Global)
    #[allow(unused_doc_comments)]
    #[wasm_bindgen(constructor, js_namespace = WebAssembly, catch)]
    pub fn new(global_descriptor: &Object, value: &JsValue) -> Result<Global, JsValue>;

    /// The value prototype property of the `WebAssembly.Global` object
    /// returns the value of the global.
    ///
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/Global)
    #[allow(unused_doc_comments)]
    #[wasm_bindgen(method, getter, structural, js_namespace = WebAssembly)]
    pub fn value(this: &Global) -> JsValue;

    #[wasm_bindgen(method, setter = value, structural, js_namespace = WebAssembly)]
    pub fn set_value(this: &Global, value: &JsValue);
}
