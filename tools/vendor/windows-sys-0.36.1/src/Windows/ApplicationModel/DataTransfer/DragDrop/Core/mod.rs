pub type CoreDragDropManager = *mut ::core::ffi::c_void;
pub type CoreDragInfo = *mut ::core::ffi::c_void;
pub type CoreDragOperation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_DataTransfer_DragDrop_Core\"`*"]
#[repr(transparent)]
pub struct CoreDragUIContentMode(pub u32);
impl CoreDragUIContentMode {
    pub const Auto: Self = Self(0u32);
    pub const Deferred: Self = Self(1u32);
}
impl ::core::marker::Copy for CoreDragUIContentMode {}
impl ::core::clone::Clone for CoreDragUIContentMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CoreDragUIOverride = *mut ::core::ffi::c_void;
pub type CoreDropOperationTargetRequestedEventArgs = *mut ::core::ffi::c_void;
pub type ICoreDropOperationTarget = *mut ::core::ffi::c_void;
