#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = GamepadEvent , extends = Event , extends = :: js_sys :: Object , js_name = GamepadButtonEvent , typescript_type = "GamepadButtonEvent")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `GamepadButtonEvent` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadButtonEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadButtonEvent`*"]
    pub type GamepadButtonEvent;
    # [wasm_bindgen (structural , method , getter , js_class = "GamepadButtonEvent" , js_name = button)]
    #[doc = "Getter for the `button` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadButtonEvent/button)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadButtonEvent`*"]
    pub fn button(this: &GamepadButtonEvent) -> u32;
    #[wasm_bindgen(catch, constructor, js_class = "GamepadButtonEvent")]
    #[doc = "The `new GamepadButtonEvent(..)` constructor, creating a new instance of `GamepadButtonEvent`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadButtonEvent/GamepadButtonEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadButtonEvent`*"]
    pub fn new(type_: &str) -> Result<GamepadButtonEvent, JsValue>;
    #[cfg(feature = "GamepadButtonEventInit")]
    #[wasm_bindgen(catch, constructor, js_class = "GamepadButtonEvent")]
    #[doc = "The `new GamepadButtonEvent(..)` constructor, creating a new instance of `GamepadButtonEvent`."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/GamepadButtonEvent/GamepadButtonEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `GamepadButtonEvent`, `GamepadButtonEventInit`*"]
    pub fn new_with_event_init_dict(
        type_: &str,
        event_init_dict: &GamepadButtonEventInit,
    ) -> Result<GamepadButtonEvent, JsValue>;
}
