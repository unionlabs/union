pub type PdfDocument = *mut ::core::ffi::c_void;
pub type PdfPage = *mut ::core::ffi::c_void;
pub type PdfPageDimensions = *mut ::core::ffi::c_void;
pub type PdfPageRenderOptions = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Data_Pdf\"`*"]
#[repr(transparent)]
pub struct PdfPageRotation(pub i32);
impl PdfPageRotation {
    pub const Normal: Self = Self(0i32);
    pub const Rotate90: Self = Self(1i32);
    pub const Rotate180: Self = Self(2i32);
    pub const Rotate270: Self = Self(3i32);
}
impl ::core::marker::Copy for PdfPageRotation {}
impl ::core::clone::Clone for PdfPageRotation {
    fn clone(&self) -> Self {
        *self
    }
}
