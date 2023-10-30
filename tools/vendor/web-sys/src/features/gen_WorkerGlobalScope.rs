#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = EventTarget , extends = :: js_sys :: Object , js_name = WorkerGlobalScope , typescript_type = "WorkerGlobalScope")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `WorkerGlobalScope` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub type WorkerGlobalScope;
    # [wasm_bindgen (structural , method , getter , js_class = "WorkerGlobalScope" , js_name = self)]
    #[doc = "Getter for the `self` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/self)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn self_(this: &WorkerGlobalScope) -> WorkerGlobalScope;
    #[cfg(feature = "WorkerLocation")]
    # [wasm_bindgen (structural , method , getter , js_class = "WorkerGlobalScope" , js_name = location)]
    #[doc = "Getter for the `location` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/location)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`, `WorkerLocation`*"]
    pub fn location(this: &WorkerGlobalScope) -> WorkerLocation;
    #[cfg(feature = "WorkerNavigator")]
    # [wasm_bindgen (structural , method , getter , js_class = "WorkerGlobalScope" , js_name = navigator)]
    #[doc = "Getter for the `navigator` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/navigator)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`, `WorkerNavigator`*"]
    pub fn navigator(this: &WorkerGlobalScope) -> WorkerNavigator;
    # [wasm_bindgen (structural , method , getter , js_class = "WorkerGlobalScope" , js_name = onerror)]
    #[doc = "Getter for the `onerror` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/onerror)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn onerror(this: &WorkerGlobalScope) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "WorkerGlobalScope" , js_name = onerror)]
    #[doc = "Setter for the `onerror` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/onerror)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_onerror(this: &WorkerGlobalScope, value: Option<&::js_sys::Function>);
    # [wasm_bindgen (structural , method , getter , js_class = "WorkerGlobalScope" , js_name = onoffline)]
    #[doc = "Getter for the `onoffline` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/onoffline)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn onoffline(this: &WorkerGlobalScope) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "WorkerGlobalScope" , js_name = onoffline)]
    #[doc = "Setter for the `onoffline` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/onoffline)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_onoffline(this: &WorkerGlobalScope, value: Option<&::js_sys::Function>);
    # [wasm_bindgen (structural , method , getter , js_class = "WorkerGlobalScope" , js_name = ononline)]
    #[doc = "Getter for the `ononline` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/ononline)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn ononline(this: &WorkerGlobalScope) -> Option<::js_sys::Function>;
    # [wasm_bindgen (structural , method , setter , js_class = "WorkerGlobalScope" , js_name = ononline)]
    #[doc = "Setter for the `ononline` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/ononline)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_ononline(this: &WorkerGlobalScope, value: Option<&::js_sys::Function>);
    #[cfg(feature = "Crypto")]
    # [wasm_bindgen (structural , catch , method , getter , js_class = "WorkerGlobalScope" , js_name = crypto)]
    #[doc = "Getter for the `crypto` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/crypto)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Crypto`, `WorkerGlobalScope`*"]
    pub fn crypto(this: &WorkerGlobalScope) -> Result<Crypto, JsValue>;
    # [wasm_bindgen (structural , method , getter , js_class = "WorkerGlobalScope" , js_name = origin)]
    #[doc = "Getter for the `origin` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/origin)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn origin(this: &WorkerGlobalScope) -> String;
    # [wasm_bindgen (structural , method , getter , js_class = "WorkerGlobalScope" , js_name = isSecureContext)]
    #[doc = "Getter for the `isSecureContext` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/isSecureContext)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn is_secure_context(this: &WorkerGlobalScope) -> bool;
    #[cfg(feature = "IdbFactory")]
    # [wasm_bindgen (structural , catch , method , getter , js_class = "WorkerGlobalScope" , js_name = indexedDB)]
    #[doc = "Getter for the `indexedDB` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/indexedDB)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `IdbFactory`, `WorkerGlobalScope`*"]
    pub fn indexed_db(this: &WorkerGlobalScope) -> Result<Option<IdbFactory>, JsValue>;
    #[cfg(feature = "CacheStorage")]
    # [wasm_bindgen (structural , catch , method , getter , js_class = "WorkerGlobalScope" , js_name = caches)]
    #[doc = "Getter for the `caches` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/caches)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CacheStorage`, `WorkerGlobalScope`*"]
    pub fn caches(this: &WorkerGlobalScope) -> Result<CacheStorage, JsValue>;
    # [wasm_bindgen (catch , method , structural , variadic , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts(this: &WorkerGlobalScope, urls: &::js_sys::Array) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts_0(this: &WorkerGlobalScope) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts_1(this: &WorkerGlobalScope, urls_1: &str) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts_2(
        this: &WorkerGlobalScope,
        urls_1: &str,
        urls_2: &str,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts_3(
        this: &WorkerGlobalScope,
        urls_1: &str,
        urls_2: &str,
        urls_3: &str,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts_4(
        this: &WorkerGlobalScope,
        urls_1: &str,
        urls_2: &str,
        urls_3: &str,
        urls_4: &str,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts_5(
        this: &WorkerGlobalScope,
        urls_1: &str,
        urls_2: &str,
        urls_3: &str,
        urls_4: &str,
        urls_5: &str,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts_6(
        this: &WorkerGlobalScope,
        urls_1: &str,
        urls_2: &str,
        urls_3: &str,
        urls_4: &str,
        urls_5: &str,
        urls_6: &str,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = importScripts)]
    #[doc = "The `importScripts()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/importScripts)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn import_scripts_7(
        this: &WorkerGlobalScope,
        urls_1: &str,
        urls_2: &str,
        urls_3: &str,
        urls_4: &str,
        urls_5: &str,
        urls_6: &str,
        urls_7: &str,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = atob)]
    #[doc = "The `atob()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/atob)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn atob(this: &WorkerGlobalScope, atob: &str) -> Result<String, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = btoa)]
    #[doc = "The `btoa()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/btoa)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn btoa(this: &WorkerGlobalScope, btoa: &str) -> Result<String, JsValue>;
    # [wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = clearInterval)]
    #[doc = "The `clearInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/clearInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn clear_interval(this: &WorkerGlobalScope);
    # [wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = clearInterval)]
    #[doc = "The `clearInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/clearInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn clear_interval_with_handle(this: &WorkerGlobalScope, handle: i32);
    # [wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = clearTimeout)]
    #[doc = "The `clearTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/clearTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn clear_timeout(this: &WorkerGlobalScope);
    # [wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = clearTimeout)]
    #[doc = "The `clearTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/clearTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn clear_timeout_with_handle(this: &WorkerGlobalScope, handle: i32);
    #[cfg(feature = "HtmlImageElement")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HtmlImageElement`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_html_image_element(
        this: &WorkerGlobalScope,
        a_image: &HtmlImageElement,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "HtmlVideoElement")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HtmlVideoElement`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_html_video_element(
        this: &WorkerGlobalScope,
        a_image: &HtmlVideoElement,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "HtmlCanvasElement")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HtmlCanvasElement`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_html_canvas_element(
        this: &WorkerGlobalScope,
        a_image: &HtmlCanvasElement,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "Blob")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Blob`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_blob(
        this: &WorkerGlobalScope,
        a_image: &Blob,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "ImageData")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ImageData`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_image_data(
        this: &WorkerGlobalScope,
        a_image: &ImageData,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "CanvasRenderingContext2d")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CanvasRenderingContext2d`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_canvas_rendering_context_2d(
        this: &WorkerGlobalScope,
        a_image: &CanvasRenderingContext2d,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "ImageBitmap")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ImageBitmap`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_image_bitmap(
        this: &WorkerGlobalScope,
        a_image: &ImageBitmap,
    ) -> Result<::js_sys::Promise, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_buffer_source(
        this: &WorkerGlobalScope,
        a_image: &::js_sys::Object,
    ) -> Result<::js_sys::Promise, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_u8_array(
        this: &WorkerGlobalScope,
        a_image: &mut [u8],
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "HtmlImageElement")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HtmlImageElement`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_html_image_element_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &HtmlImageElement,
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "HtmlVideoElement")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HtmlVideoElement`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_html_video_element_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &HtmlVideoElement,
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "HtmlCanvasElement")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `HtmlCanvasElement`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_html_canvas_element_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &HtmlCanvasElement,
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "Blob")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Blob`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_blob_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &Blob,
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "ImageData")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ImageData`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_image_data_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &ImageData,
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "CanvasRenderingContext2d")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `CanvasRenderingContext2d`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_canvas_rendering_context_2d_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &CanvasRenderingContext2d,
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "ImageBitmap")]
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `ImageBitmap`, `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_image_bitmap_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &ImageBitmap,
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_buffer_source_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &::js_sys::Object,
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = createImageBitmap)]
    #[doc = "The `createImageBitmap()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/createImageBitmap)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn create_image_bitmap_with_u8_array_and_a_sx_and_a_sy_and_a_sw_and_a_sh(
        this: &WorkerGlobalScope,
        a_image: &mut [u8],
        a_sx: i32,
        a_sy: i32,
        a_sw: i32,
        a_sh: i32,
    ) -> Result<::js_sys::Promise, JsValue>;
    #[cfg(feature = "Request")]
    # [wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/fetch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Request`, `WorkerGlobalScope`*"]
    pub fn fetch_with_request(this: &WorkerGlobalScope, input: &Request) -> ::js_sys::Promise;
    # [wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/fetch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn fetch_with_str(this: &WorkerGlobalScope, input: &str) -> ::js_sys::Promise;
    #[cfg(all(feature = "Request", feature = "RequestInit",))]
    # [wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/fetch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Request`, `RequestInit`, `WorkerGlobalScope`*"]
    pub fn fetch_with_request_and_init(
        this: &WorkerGlobalScope,
        input: &Request,
        init: &RequestInit,
    ) -> ::js_sys::Promise;
    #[cfg(feature = "RequestInit")]
    # [wasm_bindgen (method , structural , js_class = "WorkerGlobalScope" , js_name = fetch)]
    #[doc = "The `fetch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/fetch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `RequestInit`, `WorkerGlobalScope`*"]
    pub fn fetch_with_str_and_init(
        this: &WorkerGlobalScope,
        input: &str,
        init: &RequestInit,
    ) -> ::js_sys::Promise;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , variadic , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments: &::js_sys::Array,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments_0(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments_1(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments_2(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments_3(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments_4(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
        arguments_4: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments_5(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
        arguments_4: &::wasm_bindgen::JsValue,
        arguments_5: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments_6(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
        arguments_4: &::wasm_bindgen::JsValue,
        arguments_5: &::wasm_bindgen::JsValue,
        arguments_6: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_callback_and_timeout_and_arguments_7(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
        arguments_4: &::wasm_bindgen::JsValue,
        arguments_5: &::wasm_bindgen::JsValue,
        arguments_6: &::wasm_bindgen::JsValue,
        arguments_7: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str(this: &WorkerGlobalScope, handler: &str) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , variadic , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused: &::js_sys::Array,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused_0(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused_1(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused_2(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused_3(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused_4(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
        unused_4: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused_5(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
        unused_4: &::wasm_bindgen::JsValue,
        unused_5: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused_6(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
        unused_4: &::wasm_bindgen::JsValue,
        unused_5: &::wasm_bindgen::JsValue,
        unused_6: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setInterval)]
    #[doc = "The `setInterval()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setInterval)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_interval_with_str_and_timeout_and_unused_7(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
        unused_4: &::wasm_bindgen::JsValue,
        unused_5: &::wasm_bindgen::JsValue,
        unused_6: &::wasm_bindgen::JsValue,
        unused_7: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , variadic , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments: &::js_sys::Array,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments_0(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments_1(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments_2(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments_3(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments_4(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
        arguments_4: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments_5(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
        arguments_4: &::wasm_bindgen::JsValue,
        arguments_5: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments_6(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
        arguments_4: &::wasm_bindgen::JsValue,
        arguments_5: &::wasm_bindgen::JsValue,
        arguments_6: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_callback_and_timeout_and_arguments_7(
        this: &WorkerGlobalScope,
        handler: &::js_sys::Function,
        timeout: i32,
        arguments_1: &::wasm_bindgen::JsValue,
        arguments_2: &::wasm_bindgen::JsValue,
        arguments_3: &::wasm_bindgen::JsValue,
        arguments_4: &::wasm_bindgen::JsValue,
        arguments_5: &::wasm_bindgen::JsValue,
        arguments_6: &::wasm_bindgen::JsValue,
        arguments_7: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str(this: &WorkerGlobalScope, handler: &str) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , variadic , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused: &::js_sys::Array,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused_0(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused_1(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused_2(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused_3(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused_4(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
        unused_4: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused_5(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
        unused_4: &::wasm_bindgen::JsValue,
        unused_5: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused_6(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
        unused_4: &::wasm_bindgen::JsValue,
        unused_5: &::wasm_bindgen::JsValue,
        unused_6: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "WorkerGlobalScope" , js_name = setTimeout)]
    #[doc = "The `setTimeout()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/setTimeout)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `WorkerGlobalScope`*"]
    pub fn set_timeout_with_str_and_timeout_and_unused_7(
        this: &WorkerGlobalScope,
        handler: &str,
        timeout: i32,
        unused_1: &::wasm_bindgen::JsValue,
        unused_2: &::wasm_bindgen::JsValue,
        unused_3: &::wasm_bindgen::JsValue,
        unused_4: &::wasm_bindgen::JsValue,
        unused_5: &::wasm_bindgen::JsValue,
        unused_6: &::wasm_bindgen::JsValue,
        unused_7: &::wasm_bindgen::JsValue,
    ) -> Result<i32, JsValue>;
}
