pub type FrameNavigationOptions = *mut ::core::ffi::c_void;
pub type LoadCompletedEventHandler = *mut ::core::ffi::c_void;
pub type NavigatedEventHandler = *mut ::core::ffi::c_void;
pub type NavigatingCancelEventArgs = *mut ::core::ffi::c_void;
pub type NavigatingCancelEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Navigation\"`*"]
#[repr(transparent)]
pub struct NavigationCacheMode(pub i32);
impl NavigationCacheMode {
    pub const Disabled: Self = Self(0i32);
    pub const Required: Self = Self(1i32);
    pub const Enabled: Self = Self(2i32);
}
impl ::core::marker::Copy for NavigationCacheMode {}
impl ::core::clone::Clone for NavigationCacheMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type NavigationEventArgs = *mut ::core::ffi::c_void;
pub type NavigationFailedEventArgs = *mut ::core::ffi::c_void;
pub type NavigationFailedEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Navigation\"`*"]
#[repr(transparent)]
pub struct NavigationMode(pub i32);
impl NavigationMode {
    pub const New: Self = Self(0i32);
    pub const Back: Self = Self(1i32);
    pub const Forward: Self = Self(2i32);
    pub const Refresh: Self = Self(3i32);
}
impl ::core::marker::Copy for NavigationMode {}
impl ::core::clone::Clone for NavigationMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type NavigationStoppedEventHandler = *mut ::core::ffi::c_void;
pub type PageStackEntry = *mut ::core::ffi::c_void;
