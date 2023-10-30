#[cfg(feature = "Graphics_Capture")]
pub mod Capture;
#[cfg(feature = "Graphics_DirectX")]
pub mod DirectX;
#[cfg(feature = "Graphics_Display")]
pub mod Display;
#[cfg(feature = "Graphics_Effects")]
pub mod Effects;
#[cfg(feature = "Graphics_Holographic")]
pub mod Holographic;
#[cfg(feature = "Graphics_Imaging")]
pub mod Imaging;
#[cfg(feature = "Graphics_Printing")]
pub mod Printing;
#[cfg(feature = "Graphics_Printing3D")]
pub mod Printing3D;
#[repr(C)]
#[doc = "*Required features: `\"Graphics\"`*"]
pub struct DisplayAdapterId {
    pub LowPart: u32,
    pub HighPart: i32,
}
impl ::core::marker::Copy for DisplayAdapterId {}
impl ::core::clone::Clone for DisplayAdapterId {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Graphics\"`*"]
pub struct DisplayId {
    pub Value: u64,
}
impl ::core::marker::Copy for DisplayId {}
impl ::core::clone::Clone for DisplayId {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IGeometrySource2D = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Graphics\"`*"]
pub struct PointInt32 {
    pub X: i32,
    pub Y: i32,
}
impl ::core::marker::Copy for PointInt32 {}
impl ::core::clone::Clone for PointInt32 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Graphics\"`*"]
pub struct RectInt32 {
    pub X: i32,
    pub Y: i32,
    pub Width: i32,
    pub Height: i32,
}
impl ::core::marker::Copy for RectInt32 {}
impl ::core::clone::Clone for RectInt32 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Graphics\"`*"]
pub struct SizeInt32 {
    pub Width: i32,
    pub Height: i32,
}
impl ::core::marker::Copy for SizeInt32 {}
impl ::core::clone::Clone for SizeInt32 {
    fn clone(&self) -> Self {
        *self
    }
}
