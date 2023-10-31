#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = VRMockController , typescript_type = "VRMockController")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `VrMockController` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockController)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockController`*"]
    pub type VrMockController;
    # [wasm_bindgen (method , structural , js_class = "VRMockController" , js_name = newAxisMoveEvent)]
    #[doc = "The `newAxisMoveEvent()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockController/newAxisMoveEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockController`*"]
    pub fn new_axis_move_event(this: &VrMockController, axis: u32, value: f64);
    # [wasm_bindgen (method , structural , js_class = "VRMockController" , js_name = newButtonEvent)]
    #[doc = "The `newButtonEvent()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockController/newButtonEvent)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockController`*"]
    pub fn new_button_event(this: &VrMockController, button: u32, pressed: bool);
    # [wasm_bindgen (method , structural , js_class = "VRMockController" , js_name = newPoseMove)]
    #[doc = "The `newPoseMove()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockController/newPoseMove)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockController`*"]
    pub fn new_pose_move(
        this: &VrMockController,
        position: Option<&mut [f32]>,
        linear_velocity: Option<&mut [f32]>,
        linear_acceleration: Option<&mut [f32]>,
        orientation: Option<&mut [f32]>,
        angular_velocity: Option<&mut [f32]>,
        angular_acceleration: Option<&mut [f32]>,
    );
}
