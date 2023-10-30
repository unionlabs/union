pub type DesignerAppExitedEventArgs = *mut ::core::ffi::c_void;
pub type DesignerAppManager = *mut ::core::ffi::c_void;
pub type DesignerAppView = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Hosting\"`*"]
#[repr(transparent)]
pub struct DesignerAppViewState(pub i32);
impl DesignerAppViewState {
    pub const Visible: Self = Self(0i32);
    pub const Hidden: Self = Self(1i32);
}
impl ::core::marker::Copy for DesignerAppViewState {}
impl ::core::clone::Clone for DesignerAppViewState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DesktopWindowXamlSource = *mut ::core::ffi::c_void;
pub type DesktopWindowXamlSourceGotFocusEventArgs = *mut ::core::ffi::c_void;
pub type DesktopWindowXamlSourceTakeFocusRequestedEventArgs = *mut ::core::ffi::c_void;
pub type ElementCompositionPreview = *mut ::core::ffi::c_void;
pub type IXamlUIPresenterHost = *mut ::core::ffi::c_void;
pub type IXamlUIPresenterHost2 = *mut ::core::ffi::c_void;
pub type IXamlUIPresenterHost3 = *mut ::core::ffi::c_void;
pub type WindowsXamlManager = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Hosting\"`*"]
#[repr(transparent)]
pub struct XamlSourceFocusNavigationReason(pub i32);
impl XamlSourceFocusNavigationReason {
    pub const Programmatic: Self = Self(0i32);
    pub const Restore: Self = Self(1i32);
    pub const First: Self = Self(3i32);
    pub const Last: Self = Self(4i32);
    pub const Left: Self = Self(7i32);
    pub const Up: Self = Self(8i32);
    pub const Right: Self = Self(9i32);
    pub const Down: Self = Self(10i32);
}
impl ::core::marker::Copy for XamlSourceFocusNavigationReason {}
impl ::core::clone::Clone for XamlSourceFocusNavigationReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type XamlSourceFocusNavigationRequest = *mut ::core::ffi::c_void;
pub type XamlSourceFocusNavigationResult = *mut ::core::ffi::c_void;
pub type XamlUIPresenter = *mut ::core::ffi::c_void;
