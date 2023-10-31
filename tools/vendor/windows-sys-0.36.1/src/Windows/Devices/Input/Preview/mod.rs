#[doc = "*Required features: `\"Devices_Input_Preview\"`*"]
#[repr(transparent)]
pub struct GazeDeviceConfigurationStatePreview(pub i32);
impl GazeDeviceConfigurationStatePreview {
    pub const Unknown: Self = Self(0i32);
    pub const Ready: Self = Self(1i32);
    pub const Configuring: Self = Self(2i32);
    pub const ScreenSetupNeeded: Self = Self(3i32);
    pub const UserCalibrationNeeded: Self = Self(4i32);
}
impl ::core::marker::Copy for GazeDeviceConfigurationStatePreview {}
impl ::core::clone::Clone for GazeDeviceConfigurationStatePreview {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GazeDevicePreview = *mut ::core::ffi::c_void;
pub type GazeDeviceWatcherAddedPreviewEventArgs = *mut ::core::ffi::c_void;
pub type GazeDeviceWatcherPreview = *mut ::core::ffi::c_void;
pub type GazeDeviceWatcherRemovedPreviewEventArgs = *mut ::core::ffi::c_void;
pub type GazeDeviceWatcherUpdatedPreviewEventArgs = *mut ::core::ffi::c_void;
pub type GazeEnteredPreviewEventArgs = *mut ::core::ffi::c_void;
pub type GazeExitedPreviewEventArgs = *mut ::core::ffi::c_void;
pub type GazeInputSourcePreview = *mut ::core::ffi::c_void;
pub type GazeMovedPreviewEventArgs = *mut ::core::ffi::c_void;
pub type GazePointPreview = *mut ::core::ffi::c_void;
