#[repr(C)]
#[doc = "*Required features: `\"Graphics_Holographic\"`*"]
pub struct HolographicAdapterId {
    pub LowPart: u32,
    pub HighPart: i32,
}
impl ::core::marker::Copy for HolographicAdapterId {}
impl ::core::clone::Clone for HolographicAdapterId {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HolographicCamera = *mut ::core::ffi::c_void;
pub type HolographicCameraPose = *mut ::core::ffi::c_void;
pub type HolographicCameraRenderingParameters = *mut ::core::ffi::c_void;
pub type HolographicCameraViewportParameters = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Holographic\"`*"]
#[repr(transparent)]
pub struct HolographicDepthReprojectionMethod(pub i32);
impl HolographicDepthReprojectionMethod {
    pub const DepthReprojection: Self = Self(0i32);
    pub const AutoPlanar: Self = Self(1i32);
}
impl ::core::marker::Copy for HolographicDepthReprojectionMethod {}
impl ::core::clone::Clone for HolographicDepthReprojectionMethod {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HolographicDisplay = *mut ::core::ffi::c_void;
pub type HolographicFrame = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Graphics_Holographic\"`*"]
pub struct HolographicFrameId {
    pub Value: u64,
}
impl ::core::marker::Copy for HolographicFrameId {}
impl ::core::clone::Clone for HolographicFrameId {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HolographicFramePrediction = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Holographic\"`*"]
#[repr(transparent)]
pub struct HolographicFramePresentResult(pub i32);
impl HolographicFramePresentResult {
    pub const Success: Self = Self(0i32);
    pub const DeviceRemoved: Self = Self(1i32);
}
impl ::core::marker::Copy for HolographicFramePresentResult {}
impl ::core::clone::Clone for HolographicFramePresentResult {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Graphics_Holographic\"`*"]
#[repr(transparent)]
pub struct HolographicFramePresentWaitBehavior(pub i32);
impl HolographicFramePresentWaitBehavior {
    pub const WaitForFrameToFinish: Self = Self(0i32);
    pub const DoNotWaitForFrameToFinish: Self = Self(1i32);
}
impl ::core::marker::Copy for HolographicFramePresentWaitBehavior {}
impl ::core::clone::Clone for HolographicFramePresentWaitBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HolographicFramePresentationMonitor = *mut ::core::ffi::c_void;
pub type HolographicFramePresentationReport = *mut ::core::ffi::c_void;
pub type HolographicFrameRenderingReport = *mut ::core::ffi::c_void;
pub type HolographicFrameScanoutMonitor = *mut ::core::ffi::c_void;
pub type HolographicFrameScanoutReport = *mut ::core::ffi::c_void;
pub type HolographicQuadLayer = *mut ::core::ffi::c_void;
pub type HolographicQuadLayerUpdateParameters = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Holographic\"`*"]
#[repr(transparent)]
pub struct HolographicReprojectionMode(pub i32);
impl HolographicReprojectionMode {
    pub const PositionAndOrientation: Self = Self(0i32);
    pub const OrientationOnly: Self = Self(1i32);
    pub const Disabled: Self = Self(2i32);
}
impl ::core::marker::Copy for HolographicReprojectionMode {}
impl ::core::clone::Clone for HolographicReprojectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HolographicSpace = *mut ::core::ffi::c_void;
pub type HolographicSpaceCameraAddedEventArgs = *mut ::core::ffi::c_void;
pub type HolographicSpaceCameraRemovedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Holographic\"`*"]
#[repr(transparent)]
pub struct HolographicSpaceUserPresence(pub i32);
impl HolographicSpaceUserPresence {
    pub const Absent: Self = Self(0i32);
    pub const PresentPassive: Self = Self(1i32);
    pub const PresentActive: Self = Self(2i32);
}
impl ::core::marker::Copy for HolographicSpaceUserPresence {}
impl ::core::clone::Clone for HolographicSpaceUserPresence {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Graphics_Holographic\"`, `\"Foundation_Numerics\"`*"]
#[cfg(feature = "Foundation_Numerics")]
pub struct HolographicStereoTransform {
    pub Left: super::super::Foundation::Numerics::Matrix4x4,
    pub Right: super::super::Foundation::Numerics::Matrix4x4,
}
#[cfg(feature = "Foundation_Numerics")]
impl ::core::marker::Copy for HolographicStereoTransform {}
#[cfg(feature = "Foundation_Numerics")]
impl ::core::clone::Clone for HolographicStereoTransform {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HolographicViewConfiguration = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Holographic\"`*"]
#[repr(transparent)]
pub struct HolographicViewConfigurationKind(pub i32);
impl HolographicViewConfigurationKind {
    pub const Display: Self = Self(0i32);
    pub const PhotoVideoCamera: Self = Self(1i32);
}
impl ::core::marker::Copy for HolographicViewConfigurationKind {}
impl ::core::clone::Clone for HolographicViewConfigurationKind {
    fn clone(&self) -> Self {
        *self
    }
}
