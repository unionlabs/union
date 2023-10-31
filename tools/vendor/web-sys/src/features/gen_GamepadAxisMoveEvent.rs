#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = GamepadEvent , extends = Event , extends = :: js_sys :: Object , js_name = GamepadAxisMoveEvent , typescript_type = "GamepadAxisMoveEvent")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `GamepadAxisMoveEvent` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadAxisMoveEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadAxisMoveEvent`*"]
    pub type GamepadAxisMoveEvent;
    # [wasm_bindgen (structural , method , getter , js_class = "GamepadAxisMoveEvent" , js_name = axis)]
    #[doc = "Getter for the `axis` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadAxisMoveEvent/axis)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadAxisMoveEvent`*"]
    pub fn axis(this: &GamepadAxisMoveEvent) -> u32;
    # [wasm_bindgen (structural , method , getter , js_class = "GamepadAxisMoveEvent" , js_name = value)]
    #[doc = "Getter for the `value` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadAxisMoveEvent/value)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadAxisMoveEvent`*"]
    pub fn value(this: &GamepadAxisMoveEvent) -> f64;
    #[wasm_bindgen(catch, constructor, js_class = "GamepadAxisMoveEvent")]
    #[doc = "The `new GamepadAxisMoveEvent(..)` constructor, creating a new instance of `GamepadAxisMoveEvent`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadAxisMoveEvent/GamepadAxisMoveEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadAxisMoveEvent`*"]
    pub fn new(type_: &str) -> Result<GamepadAxisMoveEvent, JsValue>;
    #[cfg(feature = "GamepadAxisMoveEventInit")]
    #[wasm_bindgen(catch, constructor, js_class = "GamepadAxisMoveEvent")]
    #[doc = "The `new GamepadAxisMoveEvent(..)` constructor, creating a new instance of `GamepadAxisMoveEvent`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadAxisMoveEvent/GamepadAxisMoveEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadAxisMoveEvent`, `GamepadAxisMoveEventInit`*"]
    pub fn new_with_event_init_dict(
        type_: &str,
        event_init_dict: &GamepadAxisMoveEventInit,
    ) -> Result<GamepadAxisMoveEvent, JsValue>;
}
