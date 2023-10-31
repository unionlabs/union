#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
pub type AddPagesEventArgs = *mut ::core::ffi::c_void;
pub type AddPagesEventHandler = *mut ::core::ffi::c_void;
pub type GetPreviewPageEventArgs = *mut ::core::ffi::c_void;
pub type GetPreviewPageEventHandler = *mut ::core::ffi::c_void;
pub type PaginateEventArgs = *mut ::core::ffi::c_void;
pub type PaginateEventHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: 'UI_Xaml_Printing'*"]
#[repr(transparent)]
pub struct PreviewPageCountType(pub i32);
impl PreviewPageCountType {
    pub const Final: Self = Self(0i32);
    pub const Intermediate: Self = Self(1i32);
}
impl ::core::marker::Copy for PreviewPageCountType {}
impl ::core::clone::Clone for PreviewPageCountType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintDocument = *mut ::core::ffi::c_void;
