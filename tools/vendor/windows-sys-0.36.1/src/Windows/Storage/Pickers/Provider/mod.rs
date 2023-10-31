#[doc = "*Required features: `\"Storage_Pickers_Provider\"`*"]
#[repr(transparent)]
pub struct AddFileResult(pub i32);
impl AddFileResult {
    pub const Added: Self = Self(0i32);
    pub const AlreadyAdded: Self = Self(1i32);
    pub const NotAllowed: Self = Self(2i32);
    pub const Unavailable: Self = Self(3i32);
}
impl ::core::marker::Copy for AddFileResult {}
impl ::core::clone::Clone for AddFileResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FileOpenPickerUI = *mut ::core::ffi::c_void;
pub type FileRemovedEventArgs = *mut ::core::ffi::c_void;
pub type FileSavePickerUI = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Pickers_Provider\"`*"]
#[repr(transparent)]
pub struct FileSelectionMode(pub i32);
impl FileSelectionMode {
    pub const Single: Self = Self(0i32);
    pub const Multiple: Self = Self(1i32);
}
impl ::core::marker::Copy for FileSelectionMode {}
impl ::core::clone::Clone for FileSelectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PickerClosingDeferral = *mut ::core::ffi::c_void;
pub type PickerClosingEventArgs = *mut ::core::ffi::c_void;
pub type PickerClosingOperation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_Pickers_Provider\"`*"]
#[repr(transparent)]
pub struct SetFileNameResult(pub i32);
impl SetFileNameResult {
    pub const Succeeded: Self = Self(0i32);
    pub const NotAllowed: Self = Self(1i32);
    pub const Unavailable: Self = Self(2i32);
}
impl ::core::marker::Copy for SetFileNameResult {}
impl ::core::clone::Clone for SetFileNameResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TargetFileRequest = *mut ::core::ffi::c_void;
pub type TargetFileRequestDeferral = *mut ::core::ffi::c_void;
pub type TargetFileRequestedEventArgs = *mut ::core::ffi::c_void;
