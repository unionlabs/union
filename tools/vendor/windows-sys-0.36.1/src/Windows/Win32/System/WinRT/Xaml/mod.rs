#[doc = "*Required features: `\"Win32_System_WinRT_Xaml\"`*"]
pub const E_SURFACE_CONTENTS_LOST: u32 = 2150301728u32;
pub type IDesktopWindowXamlSourceNative = *mut ::core::ffi::c_void;
pub type IDesktopWindowXamlSourceNative2 = *mut ::core::ffi::c_void;
pub type IFindReferenceTargetsCallback = *mut ::core::ffi::c_void;
pub type IReferenceTracker = *mut ::core::ffi::c_void;
pub type IReferenceTrackerExtension = *mut ::core::ffi::c_void;
pub type IReferenceTrackerHost = *mut ::core::ffi::c_void;
pub type IReferenceTrackerManager = *mut ::core::ffi::c_void;
pub type IReferenceTrackerTarget = *mut ::core::ffi::c_void;
pub type ISurfaceImageSourceManagerNative = *mut ::core::ffi::c_void;
pub type ISurfaceImageSourceNative = *mut ::core::ffi::c_void;
pub type ISurfaceImageSourceNativeWithD2D = *mut ::core::ffi::c_void;
pub type ISwapChainBackgroundPanelNative = *mut ::core::ffi::c_void;
pub type ISwapChainPanelNative = *mut ::core::ffi::c_void;
pub type ISwapChainPanelNative2 = *mut ::core::ffi::c_void;
pub type ITrackerOwner = *mut ::core::ffi::c_void;
pub type IVirtualSurfaceImageSourceNative = *mut ::core::ffi::c_void;
pub type IVirtualSurfaceUpdatesCallbackNative = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_WinRT_Xaml\"`*"]
pub struct TrackerHandle__ {
    pub unused: i32,
}
impl ::core::marker::Copy for TrackerHandle__ {}
impl ::core::clone::Clone for TrackerHandle__ {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_System_WinRT_Xaml\"`*"]
pub type XAML_REFERENCETRACKER_DISCONNECT = i32;
#[doc = "*Required features: `\"Win32_System_WinRT_Xaml\"`*"]
pub const XAML_REFERENCETRACKER_DISCONNECT_DEFAULT: XAML_REFERENCETRACKER_DISCONNECT = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT_Xaml\"`*"]
pub const XAML_REFERENCETRACKER_DISCONNECT_SUSPEND: XAML_REFERENCETRACKER_DISCONNECT = 1i32;
