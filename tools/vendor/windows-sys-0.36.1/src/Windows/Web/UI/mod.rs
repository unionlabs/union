#[cfg(feature = "Web_UI_Interop")]
pub mod Interop;
pub type IWebViewControl = *mut ::core::ffi::c_void;
pub type IWebViewControl2 = *mut ::core::ffi::c_void;
pub type WebViewControlContentLoadingEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlDOMContentLoadedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlDeferredPermissionRequest = *mut ::core::ffi::c_void;
pub type WebViewControlLongRunningScriptDetectedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlNavigationCompletedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlNavigationStartingEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlNewWindowRequestedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlPermissionRequest = *mut ::core::ffi::c_void;
pub type WebViewControlPermissionRequestedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Web_UI\"`*"]
#[repr(transparent)]
pub struct WebViewControlPermissionState(pub i32);
impl WebViewControlPermissionState {
    pub const Unknown: Self = Self(0i32);
    pub const Defer: Self = Self(1i32);
    pub const Allow: Self = Self(2i32);
    pub const Deny: Self = Self(3i32);
}
impl ::core::marker::Copy for WebViewControlPermissionState {}
impl ::core::clone::Clone for WebViewControlPermissionState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Web_UI\"`*"]
#[repr(transparent)]
pub struct WebViewControlPermissionType(pub i32);
impl WebViewControlPermissionType {
    pub const Geolocation: Self = Self(0i32);
    pub const UnlimitedIndexedDBQuota: Self = Self(1i32);
    pub const Media: Self = Self(2i32);
    pub const PointerLock: Self = Self(3i32);
    pub const WebNotifications: Self = Self(4i32);
    pub const Screen: Self = Self(5i32);
    pub const ImmersiveView: Self = Self(6i32);
}
impl ::core::marker::Copy for WebViewControlPermissionType {}
impl ::core::clone::Clone for WebViewControlPermissionType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebViewControlScriptNotifyEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlSettings = *mut ::core::ffi::c_void;
pub type WebViewControlUnsupportedUriSchemeIdentifiedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlUnviewableContentIdentifiedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlWebResourceRequestedEventArgs = *mut ::core::ffi::c_void;
