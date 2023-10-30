#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DCompositionAttachMouseDragToHwnd(visual: IDCompositionVisual, hwnd: super::super::Foundation::HWND, enable: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DCompositionAttachMouseWheelToHwnd(visual: IDCompositionVisual, hwnd: super::super::Foundation::HWND, enable: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DCompositionBoostCompositorClock(enable: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Graphics_Dxgi\"`*"]
    #[cfg(feature = "Win32_Graphics_Dxgi")]
    pub fn DCompositionCreateDevice(dxgidevice: super::Dxgi::IDXGIDevice, iid: *const ::windows_sys::core::GUID, dcompositiondevice: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
    pub fn DCompositionCreateDevice2(renderingdevice: ::windows_sys::core::IUnknown, iid: *const ::windows_sys::core::GUID, dcompositiondevice: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
    pub fn DCompositionCreateDevice3(renderingdevice: ::windows_sys::core::IUnknown, iid: *const ::windows_sys::core::GUID, dcompositiondevice: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn DCompositionCreateSurfaceHandle(desiredaccess: u32, securityattributes: *const super::super::Security::SECURITY_ATTRIBUTES, surfacehandle: *mut super::super::Foundation::HANDLE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
    pub fn DCompositionGetFrameId(frameidtype: COMPOSITION_FRAME_ID_TYPE, frameid: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DCompositionGetStatistics(frameid: u64, framestats: *mut COMPOSITION_FRAME_STATS, targetidcount: u32, targetids: *mut COMPOSITION_TARGET_ID, actualtargetidcount: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DCompositionGetTargetStatistics(frameid: u64, targetid: *const COMPOSITION_TARGET_ID, targetstats: *mut COMPOSITION_TARGET_STATS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DCompositionWaitForCompositorClock(count: u32, handles: *const super::super::Foundation::HANDLE, timeoutinms: u32) -> u32;
}
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const COMPOSITIONOBJECT_READ: i32 = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const COMPOSITIONOBJECT_WRITE: i32 = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub type COMPOSITION_FRAME_ID_TYPE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const COMPOSITION_FRAME_ID_CREATED: COMPOSITION_FRAME_ID_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const COMPOSITION_FRAME_ID_CONFIRMED: COMPOSITION_FRAME_ID_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const COMPOSITION_FRAME_ID_COMPLETED: COMPOSITION_FRAME_ID_TYPE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub struct COMPOSITION_FRAME_STATS {
    pub startTime: u64,
    pub targetTime: u64,
    pub framePeriod: u64,
}
impl ::core::marker::Copy for COMPOSITION_FRAME_STATS {}
impl ::core::clone::Clone for COMPOSITION_FRAME_STATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub struct COMPOSITION_STATS {
    pub presentCount: u32,
    pub refreshCount: u32,
    pub virtualRefreshCount: u32,
    pub time: u64,
}
impl ::core::marker::Copy for COMPOSITION_STATS {}
impl ::core::clone::Clone for COMPOSITION_STATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const COMPOSITION_STATS_MAX_TARGETS: u32 = 256u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct COMPOSITION_TARGET_ID {
    pub displayAdapterLuid: super::super::Foundation::LUID,
    pub renderAdapterLuid: super::super::Foundation::LUID,
    pub vidPnSourceId: u32,
    pub vidPnTargetId: u32,
    pub uniqueId: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for COMPOSITION_TARGET_ID {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for COMPOSITION_TARGET_ID {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub struct COMPOSITION_TARGET_STATS {
    pub outstandingPresents: u32,
    pub presentTime: u64,
    pub vblankDuration: u64,
    pub presentedStats: COMPOSITION_STATS,
    pub completedStats: COMPOSITION_STATS,
}
impl ::core::marker::Copy for COMPOSITION_TARGET_STATS {}
impl ::core::clone::Clone for COMPOSITION_TARGET_STATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub type DCOMPOSITION_BACKFACE_VISIBILITY = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BACKFACE_VISIBILITY_VISIBLE: DCOMPOSITION_BACKFACE_VISIBILITY = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BACKFACE_VISIBILITY_HIDDEN: DCOMPOSITION_BACKFACE_VISIBILITY = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BACKFACE_VISIBILITY_INHERIT: DCOMPOSITION_BACKFACE_VISIBILITY = -1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub type DCOMPOSITION_BITMAP_INTERPOLATION_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR: DCOMPOSITION_BITMAP_INTERPOLATION_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR: DCOMPOSITION_BITMAP_INTERPOLATION_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BITMAP_INTERPOLATION_MODE_INHERIT: DCOMPOSITION_BITMAP_INTERPOLATION_MODE = -1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub type DCOMPOSITION_BORDER_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BORDER_MODE_SOFT: DCOMPOSITION_BORDER_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BORDER_MODE_HARD: DCOMPOSITION_BORDER_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_BORDER_MODE_INHERIT: DCOMPOSITION_BORDER_MODE = -1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub type DCOMPOSITION_COMPOSITE_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_COMPOSITE_MODE_SOURCE_OVER: DCOMPOSITION_COMPOSITE_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_COMPOSITE_MODE_DESTINATION_INVERT: DCOMPOSITION_COMPOSITE_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_COMPOSITE_MODE_MIN_BLEND: DCOMPOSITION_COMPOSITE_MODE = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_COMPOSITE_MODE_INHERIT: DCOMPOSITION_COMPOSITE_MODE = -1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub type DCOMPOSITION_DEPTH_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_DEPTH_MODE_TREE: DCOMPOSITION_DEPTH_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_DEPTH_MODE_SPATIAL: DCOMPOSITION_DEPTH_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_DEPTH_MODE_SORTED: DCOMPOSITION_DEPTH_MODE = 3i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_DEPTH_MODE_INHERIT: DCOMPOSITION_DEPTH_MODE = -1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`, `\"Win32_Graphics_Dxgi_Common\"`*"]
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
pub struct DCOMPOSITION_FRAME_STATISTICS {
    pub lastFrameTime: i64,
    pub currentCompositionRate: super::Dxgi::Common::DXGI_RATIONAL,
    pub currentTime: i64,
    pub timeFrequency: i64,
    pub nextEstimatedFrameTime: i64,
}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::marker::Copy for DCOMPOSITION_FRAME_STATISTICS {}
#[cfg(feature = "Win32_Graphics_Dxgi_Common")]
impl ::core::clone::Clone for DCOMPOSITION_FRAME_STATISTICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_MAX_WAITFORCOMPOSITORCLOCK_OBJECTS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub type DCOMPOSITION_OPACITY_MODE = i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_OPACITY_MODE_LAYER: DCOMPOSITION_OPACITY_MODE = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_OPACITY_MODE_MULTIPLY: DCOMPOSITION_OPACITY_MODE = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub const DCOMPOSITION_OPACITY_MODE_INHERIT: DCOMPOSITION_OPACITY_MODE = -1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_Graphics_DirectComposition\"`*"]
pub struct DCompositionInkTrailPoint {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}
impl ::core::marker::Copy for DCompositionInkTrailPoint {}
impl ::core::clone::Clone for DCompositionInkTrailPoint {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IDCompositionAffineTransform2DEffect = *mut ::core::ffi::c_void;
pub type IDCompositionAnimation = *mut ::core::ffi::c_void;
pub type IDCompositionArithmeticCompositeEffect = *mut ::core::ffi::c_void;
pub type IDCompositionBlendEffect = *mut ::core::ffi::c_void;
pub type IDCompositionBrightnessEffect = *mut ::core::ffi::c_void;
pub type IDCompositionClip = *mut ::core::ffi::c_void;
pub type IDCompositionColorMatrixEffect = *mut ::core::ffi::c_void;
pub type IDCompositionCompositeEffect = *mut ::core::ffi::c_void;
pub type IDCompositionDelegatedInkTrail = *mut ::core::ffi::c_void;
pub type IDCompositionDesktopDevice = *mut ::core::ffi::c_void;
pub type IDCompositionDevice = *mut ::core::ffi::c_void;
pub type IDCompositionDevice2 = *mut ::core::ffi::c_void;
pub type IDCompositionDevice3 = *mut ::core::ffi::c_void;
pub type IDCompositionDeviceDebug = *mut ::core::ffi::c_void;
pub type IDCompositionEffect = *mut ::core::ffi::c_void;
pub type IDCompositionEffectGroup = *mut ::core::ffi::c_void;
pub type IDCompositionFilterEffect = *mut ::core::ffi::c_void;
pub type IDCompositionGaussianBlurEffect = *mut ::core::ffi::c_void;
pub type IDCompositionHueRotationEffect = *mut ::core::ffi::c_void;
pub type IDCompositionInkTrailDevice = *mut ::core::ffi::c_void;
pub type IDCompositionLinearTransferEffect = *mut ::core::ffi::c_void;
pub type IDCompositionMatrixTransform = *mut ::core::ffi::c_void;
pub type IDCompositionMatrixTransform3D = *mut ::core::ffi::c_void;
pub type IDCompositionRectangleClip = *mut ::core::ffi::c_void;
pub type IDCompositionRotateTransform = *mut ::core::ffi::c_void;
pub type IDCompositionRotateTransform3D = *mut ::core::ffi::c_void;
pub type IDCompositionSaturationEffect = *mut ::core::ffi::c_void;
pub type IDCompositionScaleTransform = *mut ::core::ffi::c_void;
pub type IDCompositionScaleTransform3D = *mut ::core::ffi::c_void;
pub type IDCompositionShadowEffect = *mut ::core::ffi::c_void;
pub type IDCompositionSkewTransform = *mut ::core::ffi::c_void;
pub type IDCompositionSurface = *mut ::core::ffi::c_void;
pub type IDCompositionSurfaceFactory = *mut ::core::ffi::c_void;
pub type IDCompositionTableTransferEffect = *mut ::core::ffi::c_void;
pub type IDCompositionTarget = *mut ::core::ffi::c_void;
pub type IDCompositionTransform = *mut ::core::ffi::c_void;
pub type IDCompositionTransform3D = *mut ::core::ffi::c_void;
pub type IDCompositionTranslateTransform = *mut ::core::ffi::c_void;
pub type IDCompositionTranslateTransform3D = *mut ::core::ffi::c_void;
pub type IDCompositionTurbulenceEffect = *mut ::core::ffi::c_void;
pub type IDCompositionVirtualSurface = *mut ::core::ffi::c_void;
pub type IDCompositionVisual = *mut ::core::ffi::c_void;
pub type IDCompositionVisual2 = *mut ::core::ffi::c_void;
pub type IDCompositionVisual3 = *mut ::core::ffi::c_void;
pub type IDCompositionVisualDebug = *mut ::core::ffi::c_void;
