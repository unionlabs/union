pub type IUICommand = *mut ::core::ffi::c_void;
pub type MessageDialog = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Popups\"`*"]
#[repr(transparent)]
pub struct MessageDialogOptions(pub u32);
impl MessageDialogOptions {
    pub const None: Self = Self(0u32);
    pub const AcceptUserInputAfterDelay: Self = Self(1u32);
}
impl ::core::marker::Copy for MessageDialogOptions {}
impl ::core::clone::Clone for MessageDialogOptions {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Popups\"`*"]
#[repr(transparent)]
pub struct Placement(pub i32);
impl Placement {
    pub const Default: Self = Self(0i32);
    pub const Above: Self = Self(1i32);
    pub const Below: Self = Self(2i32);
    pub const Left: Self = Self(3i32);
    pub const Right: Self = Self(4i32);
}
impl ::core::marker::Copy for Placement {}
impl ::core::clone::Clone for Placement {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PopupMenu = *mut ::core::ffi::c_void;
pub type UICommand = *mut ::core::ffi::c_void;
pub type UICommandInvokedHandler = *mut ::core::ffi::c_void;
pub type UICommandSeparator = *mut ::core::ffi::c_void;
