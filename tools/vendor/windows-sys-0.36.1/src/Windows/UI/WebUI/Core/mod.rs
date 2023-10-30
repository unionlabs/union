pub type IWebUICommandBarElement = *mut ::core::ffi::c_void;
pub type IWebUICommandBarIcon = *mut ::core::ffi::c_void;
pub type MenuClosedEventHandler = *mut ::core::ffi::c_void;
pub type MenuOpenedEventHandler = *mut ::core::ffi::c_void;
pub type SizeChangedEventHandler = *mut ::core::ffi::c_void;
pub type WebUICommandBar = *mut ::core::ffi::c_void;
pub type WebUICommandBarBitmapIcon = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_WebUI_Core\"`*"]
#[repr(transparent)]
pub struct WebUICommandBarClosedDisplayMode(pub i32);
impl WebUICommandBarClosedDisplayMode {
    pub const Default: Self = Self(0i32);
    pub const Minimal: Self = Self(1i32);
    pub const Compact: Self = Self(2i32);
}
impl ::core::marker::Copy for WebUICommandBarClosedDisplayMode {}
impl ::core::clone::Clone for WebUICommandBarClosedDisplayMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebUICommandBarConfirmationButton = *mut ::core::ffi::c_void;
pub type WebUICommandBarIconButton = *mut ::core::ffi::c_void;
pub type WebUICommandBarItemInvokedEventArgs = *mut ::core::ffi::c_void;
pub type WebUICommandBarSizeChangedEventArgs = *mut ::core::ffi::c_void;
pub type WebUICommandBarSymbolIcon = *mut ::core::ffi::c_void;
