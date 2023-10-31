pub type CompositeTransform3D = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"UI_Xaml_Media_Media3D\"`*"]
pub struct Matrix3D {
    pub M11: f64,
    pub M12: f64,
    pub M13: f64,
    pub M14: f64,
    pub M21: f64,
    pub M22: f64,
    pub M23: f64,
    pub M24: f64,
    pub M31: f64,
    pub M32: f64,
    pub M33: f64,
    pub M34: f64,
    pub OffsetX: f64,
    pub OffsetY: f64,
    pub OffsetZ: f64,
    pub M44: f64,
}
impl ::core::marker::Copy for Matrix3D {}
impl ::core::clone::Clone for Matrix3D {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Matrix3DHelper = *mut ::core::ffi::c_void;
pub type PerspectiveTransform3D = *mut ::core::ffi::c_void;
pub type Transform3D = *mut ::core::ffi::c_void;
