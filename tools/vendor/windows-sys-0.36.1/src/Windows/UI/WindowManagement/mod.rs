#[cfg(feature = "UI_WindowManagement_Preview")]
pub mod Preview;
pub type AppWindow = *mut ::core::ffi::c_void;
pub type AppWindowChangedEventArgs = *mut ::core::ffi::c_void;
pub type AppWindowCloseRequestedEventArgs = *mut ::core::ffi::c_void;
pub type AppWindowClosedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_WindowManagement\"`*"]
#[repr(transparent)]
pub struct AppWindowClosedReason(pub i32);
impl AppWindowClosedReason {
    pub const Other: Self = Self(0i32);
    pub const AppInitiated: Self = Self(1i32);
    pub const UserInitiated: Self = Self(2i32);
}
impl ::core::marker::Copy for AppWindowClosedReason {}
impl ::core::clone::Clone for AppWindowClosedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppWindowFrame = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_WindowManagement\"`*"]
#[repr(transparent)]
pub struct AppWindowFrameStyle(pub i32);
impl AppWindowFrameStyle {
    pub const Default: Self = Self(0i32);
    pub const NoFrame: Self = Self(1i32);
}
impl ::core::marker::Copy for AppWindowFrameStyle {}
impl ::core::clone::Clone for AppWindowFrameStyle {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppWindowPlacement = *mut ::core::ffi::c_void;
pub type AppWindowPresentationConfiguration = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_WindowManagement\"`*"]
#[repr(transparent)]
pub struct AppWindowPresentationKind(pub i32);
impl AppWindowPresentationKind {
    pub const Default: Self = Self(0i32);
    pub const CompactOverlay: Self = Self(1i32);
    pub const FullScreen: Self = Self(2i32);
}
impl ::core::marker::Copy for AppWindowPresentationKind {}
impl ::core::clone::Clone for AppWindowPresentationKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AppWindowPresenter = *mut ::core::ffi::c_void;
pub type AppWindowTitleBar = *mut ::core::ffi::c_void;
pub type AppWindowTitleBarOcclusion = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_WindowManagement\"`*"]
#[repr(transparent)]
pub struct AppWindowTitleBarVisibility(pub i32);
impl AppWindowTitleBarVisibility {
    pub const Default: Self = Self(0i32);
    pub const AlwaysHidden: Self = Self(1i32);
}
impl ::core::marker::Copy for AppWindowTitleBarVisibility {}
impl ::core::clone::Clone for AppWindowTitleBarVisibility {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CompactOverlayPresentationConfiguration = *mut ::core::ffi::c_void;
pub type DefaultPresentationConfiguration = *mut ::core::ffi::c_void;
pub type DisplayRegion = *mut ::core::ffi::c_void;
pub type FullScreenPresentationConfiguration = *mut ::core::ffi::c_void;
pub type WindowingEnvironment = *mut ::core::ffi::c_void;
pub type WindowingEnvironmentAddedEventArgs = *mut ::core::ffi::c_void;
pub type WindowingEnvironmentChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_WindowManagement\"`*"]
#[repr(transparent)]
pub struct WindowingEnvironmentKind(pub i32);
impl WindowingEnvironmentKind {
    pub const Unknown: Self = Self(0i32);
    pub const Overlapped: Self = Self(1i32);
    pub const Tiled: Self = Self(2i32);
}
impl ::core::marker::Copy for WindowingEnvironmentKind {}
impl ::core::clone::Clone for WindowingEnvironmentKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WindowingEnvironmentRemovedEventArgs = *mut ::core::ffi::c_void;
