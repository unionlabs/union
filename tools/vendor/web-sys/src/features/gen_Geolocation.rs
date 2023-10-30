#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (is_type_of = | _ | false , extends = :: js_sys :: Object , js_name = Geolocation , typescript_type = "Geolocation")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `Geolocation` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Geolocation)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Geolocation`*"]
    pub type Geolocation;
    # [wasm_bindgen (method , structural , js_class = "Geolocation" , js_name = clearWatch)]
    #[doc = "The `clearWatch()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Geolocation/clearWatch)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Geolocation`*"]
    pub fn clear_watch(this: &Geolocation, watch_id: i32);
    # [wasm_bindgen (catch , method , structural , js_class = "Geolocation" , js_name = getCurrentPosition)]
    #[doc = "The `getCurrentPosition()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Geolocation/getCurrentPosition)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Geolocation`*"]
    pub fn get_current_position(
        this: &Geolocation,
        success_callback: &::js_sys::Function,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "Geolocation" , js_name = getCurrentPosition)]
    #[doc = "The `getCurrentPosition()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Geolocation/getCurrentPosition)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Geolocation`*"]
    pub fn get_current_position_with_error_callback(
        this: &Geolocation,
        success_callback: &::js_sys::Function,
        error_callback: Option<&::js_sys::Function>,
    ) -> Result<(), JsValue>;
    #[cfg(feature = "PositionOptions")]
    # [wasm_bindgen (catch , method , structural , js_class = "Geolocation" , js_name = getCurrentPosition)]
    #[doc = "The `getCurrentPosition()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Geolocation/getCurrentPosition)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Geolocation`, `PositionOptions`*"]
    pub fn get_current_position_with_error_callback_and_options(
        this: &Geolocation,
        success_callback: &::js_sys::Function,
        error_callback: Option<&::js_sys::Function>,
        options: &PositionOptions,
    ) -> Result<(), JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "Geolocation" , js_name = watchPosition)]
    #[doc = "The `watchPosition()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Geolocation/watchPosition)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Geolocation`*"]
    pub fn watch_position(
        this: &Geolocation,
        success_callback: &::js_sys::Function,
    ) -> Result<i32, JsValue>;
    # [wasm_bindgen (catch , method , structural , js_class = "Geolocation" , js_name = watchPosition)]
    #[doc = "The `watchPosition()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Geolocation/watchPosition)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Geolocation`*"]
    pub fn watch_position_with_error_callback(
        this: &Geolocation,
        success_callback: &::js_sys::Function,
        error_callback: Option<&::js_sys::Function>,
    ) -> Result<i32, JsValue>;
    #[cfg(feature = "PositionOptions")]
    # [wasm_bindgen (catch , method , structural , js_class = "Geolocation" , js_name = watchPosition)]
    #[doc = "The `watchPosition()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Geolocation/watchPosition)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Geolocation`, `PositionOptions`*"]
    pub fn watch_position_with_error_callback_and_options(
        this: &Geolocation,
        success_callback: &::js_sys::Function,
        error_callback: Option<&::js_sys::Function>,
        options: &PositionOptions,
    ) -> Result<i32, JsValue>;
}
