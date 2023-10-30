pub type CoreFrameworkInputView = *mut ::core::ffi::c_void;
pub type CoreFrameworkInputViewAnimationStartingEventArgs = *mut ::core::ffi::c_void;
pub type CoreFrameworkInputViewOcclusionsChangedEventArgs = *mut ::core::ffi::c_void;
pub type CoreInputView = *mut ::core::ffi::c_void;
pub type CoreInputViewAnimationStartingEventArgs = *mut ::core::ffi::c_void;
pub type CoreInputViewHidingEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement_Core\"`*"]
#[repr(transparent)]
pub struct CoreInputViewKind(pub i32);
impl CoreInputViewKind {
    pub const Default: Self = Self(0i32);
    pub const Keyboard: Self = Self(1i32);
    pub const Handwriting: Self = Self(2i32);
    pub const Emoji: Self = Self(3i32);
    pub const Symbols: Self = Self(4i32);
    pub const Clipboard: Self = Self(5i32);
    pub const Dictation: Self = Self(6i32);
}
impl ::core::marker::Copy for CoreInputViewKind {}
impl ::core::clone::Clone for CoreInputViewKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CoreInputViewOcclusion = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement_Core\"`*"]
#[repr(transparent)]
pub struct CoreInputViewOcclusionKind(pub i32);
impl CoreInputViewOcclusionKind {
    pub const Docked: Self = Self(0i32);
    pub const Floating: Self = Self(1i32);
    pub const Overlay: Self = Self(2i32);
}
impl ::core::marker::Copy for CoreInputViewOcclusionKind {}
impl ::core::clone::Clone for CoreInputViewOcclusionKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CoreInputViewOcclusionsChangedEventArgs = *mut ::core::ffi::c_void;
pub type CoreInputViewShowingEventArgs = *mut ::core::ffi::c_void;
pub type CoreInputViewTransferringXYFocusEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_ViewManagement_Core\"`*"]
#[repr(transparent)]
pub struct CoreInputViewXYFocusTransferDirection(pub i32);
impl CoreInputViewXYFocusTransferDirection {
    pub const Up: Self = Self(0i32);
    pub const Right: Self = Self(1i32);
    pub const Down: Self = Self(2i32);
    pub const Left: Self = Self(3i32);
}
impl ::core::marker::Copy for CoreInputViewXYFocusTransferDirection {}
impl ::core::clone::Clone for CoreInputViewXYFocusTransferDirection {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UISettingsController = *mut ::core::ffi::c_void;
