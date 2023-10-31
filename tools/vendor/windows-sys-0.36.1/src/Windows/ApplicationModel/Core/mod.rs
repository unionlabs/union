pub type AppListEntry = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Core\"`*"]
#[repr(transparent)]
pub struct AppRestartFailureReason(pub i32);
impl AppRestartFailureReason {
    pub const RestartPending: Self = Self(0i32);
    pub const NotInForeground: Self = Self(1i32);
    pub const InvalidUser: Self = Self(2i32);
    pub const Other: Self = Self(3i32);
}
impl ::core::marker::Copy for AppRestartFailureReason {}
impl ::core::clone::Clone for AppRestartFailureReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CoreApplicationView = *mut ::core::ffi::c_void;
pub type CoreApplicationViewTitleBar = *mut ::core::ffi::c_void;
pub type HostedViewClosingEventArgs = *mut ::core::ffi::c_void;
pub type ICoreApplicationUnhandledError = *mut ::core::ffi::c_void;
pub type IFrameworkView = *mut ::core::ffi::c_void;
pub type IFrameworkViewSource = *mut ::core::ffi::c_void;
pub type UnhandledError = *mut ::core::ffi::c_void;
pub type UnhandledErrorDetectedEventArgs = *mut ::core::ffi::c_void;
