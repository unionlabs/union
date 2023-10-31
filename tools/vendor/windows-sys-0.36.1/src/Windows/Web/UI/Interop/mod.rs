pub type WebViewControl = *mut ::core::ffi::c_void;
pub type WebViewControlAcceleratorKeyPressedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Web_UI_Interop\"`*"]
#[repr(transparent)]
pub struct WebViewControlAcceleratorKeyRoutingStage(pub i32);
impl WebViewControlAcceleratorKeyRoutingStage {
    pub const Tunneling: Self = Self(0i32);
    pub const Bubbling: Self = Self(1i32);
}
impl ::core::marker::Copy for WebViewControlAcceleratorKeyRoutingStage {}
impl ::core::clone::Clone for WebViewControlAcceleratorKeyRoutingStage {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Web_UI_Interop\"`*"]
#[repr(transparent)]
pub struct WebViewControlMoveFocusReason(pub i32);
impl WebViewControlMoveFocusReason {
    pub const Programmatic: Self = Self(0i32);
    pub const Next: Self = Self(1i32);
    pub const Previous: Self = Self(2i32);
}
impl ::core::marker::Copy for WebViewControlMoveFocusReason {}
impl ::core::clone::Clone for WebViewControlMoveFocusReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebViewControlMoveFocusRequestedEventArgs = *mut ::core::ffi::c_void;
pub type WebViewControlProcess = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Web_UI_Interop\"`*"]
#[repr(transparent)]
pub struct WebViewControlProcessCapabilityState(pub i32);
impl WebViewControlProcessCapabilityState {
    pub const Default: Self = Self(0i32);
    pub const Disabled: Self = Self(1i32);
    pub const Enabled: Self = Self(2i32);
}
impl ::core::marker::Copy for WebViewControlProcessCapabilityState {}
impl ::core::clone::Clone for WebViewControlProcessCapabilityState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebViewControlProcessOptions = *mut ::core::ffi::c_void;
