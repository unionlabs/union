pub type CameraIntrinsics = *mut ::core::ffi::c_void;
pub type DepthCorrelatedCoordinateMapper = *mut ::core::ffi::c_void;
pub type FrameControlCapabilities = *mut ::core::ffi::c_void;
pub type FrameController = *mut ::core::ffi::c_void;
pub type FrameExposureCapabilities = *mut ::core::ffi::c_void;
pub type FrameExposureCompensationCapabilities = *mut ::core::ffi::c_void;
pub type FrameExposureCompensationControl = *mut ::core::ffi::c_void;
pub type FrameExposureControl = *mut ::core::ffi::c_void;
pub type FrameFlashCapabilities = *mut ::core::ffi::c_void;
pub type FrameFlashControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices_Core\"`*"]
#[repr(transparent)]
pub struct FrameFlashMode(pub i32);
impl FrameFlashMode {
    pub const Disable: Self = Self(0i32);
    pub const Enable: Self = Self(1i32);
    pub const Global: Self = Self(2i32);
}
impl ::core::marker::Copy for FrameFlashMode {}
impl ::core::clone::Clone for FrameFlashMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FrameFocusCapabilities = *mut ::core::ffi::c_void;
pub type FrameFocusControl = *mut ::core::ffi::c_void;
pub type FrameIsoSpeedCapabilities = *mut ::core::ffi::c_void;
pub type FrameIsoSpeedControl = *mut ::core::ffi::c_void;
pub type VariablePhotoSequenceController = *mut ::core::ffi::c_void;
