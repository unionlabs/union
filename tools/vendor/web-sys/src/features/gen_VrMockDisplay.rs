#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = :: js_sys :: Object , js_name = VRMockDisplay , typescript_type = "VRMockDisplay")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `VrMockDisplay` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockDisplay)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockDisplay`*"]
    pub type VrMockDisplay;
    #[cfg(feature = "VrEye")]
    # [wasm_bindgen (method , structural , js_class = "VRMockDisplay" , js_name = setEyeParameter)]
    #[doc = "The `setEyeParameter()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockDisplay/setEyeParameter)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrEye`, `VrMockDisplay`*"]
    pub fn set_eye_parameter(
        this: &VrMockDisplay,
        eye: VrEye,
        offset_x: f64,
        offset_y: f64,
        offset_z: f64,
        up_degree: f64,
        right_degree: f64,
        down_degree: f64,
        left_degree: f64,
    );
    # [wasm_bindgen (method , structural , js_class = "VRMockDisplay" , js_name = setEyeResolution)]
    #[doc = "The `setEyeResolution()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockDisplay/setEyeResolution)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockDisplay`*"]
    pub fn set_eye_resolution(this: &VrMockDisplay, a_render_width: u32, a_render_height: u32);
    # [wasm_bindgen (method , structural , js_class = "VRMockDisplay" , js_name = setMountState)]
    #[doc = "The `setMountState()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockDisplay/setMountState)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockDisplay`*"]
    pub fn set_mount_state(this: &VrMockDisplay, is_mounted: bool);
    # [wasm_bindgen (method , structural , js_class = "VRMockDisplay" , js_name = setPose)]
    #[doc = "The `setPose()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockDisplay/setPose)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockDisplay`*"]
    pub fn set_pose(
        this: &VrMockDisplay,
        position: Option<&mut [f32]>,
        linear_velocity: Option<&mut [f32]>,
        linear_acceleration: Option<&mut [f32]>,
        orientation: Option<&mut [f32]>,
        angular_velocity: Option<&mut [f32]>,
        angular_acceleration: Option<&mut [f32]>,
    );
    # [wasm_bindgen (method , structural , js_class = "VRMockDisplay" , js_name = update)]
    #[doc = "The `update()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/VRMockDisplay/update)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `VrMockDisplay`*"]
    pub fn update(this: &VrMockDisplay);
}
