#[doc = "*Required features: `\"UI_Xaml_Media_Imaging\"`*"]
#[repr(transparent)]
pub struct BitmapCreateOptions(pub u32);
impl BitmapCreateOptions {
    pub const None: Self = Self(0u32);
    pub const IgnoreImageCache: Self = Self(8u32);
}
impl ::core::marker::Copy for BitmapCreateOptions {}
impl ::core::clone::Clone for BitmapCreateOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BitmapImage = *mut ::core::ffi::c_void;
pub type BitmapSource = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media_Imaging\"`*"]
#[repr(transparent)]
pub struct DecodePixelType(pub i32);
impl DecodePixelType {
    pub const Physical: Self = Self(0i32);
    pub const Logical: Self = Self(1i32);
}
impl ::core::marker::Copy for DecodePixelType {}
impl ::core::clone::Clone for DecodePixelType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DownloadProgressEventArgs = *mut ::core::ffi::c_void;
pub type DownloadProgressEventHandler = *mut ::core::ffi::c_void;
pub type RenderTargetBitmap = *mut ::core::ffi::c_void;
pub type SoftwareBitmapSource = *mut ::core::ffi::c_void;
pub type SurfaceImageSource = *mut ::core::ffi::c_void;
pub type SvgImageSource = *mut ::core::ffi::c_void;
pub type SvgImageSourceFailedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Xaml_Media_Imaging\"`*"]
#[repr(transparent)]
pub struct SvgImageSourceLoadStatus(pub i32);
impl SvgImageSourceLoadStatus {
    pub const Success: Self = Self(0i32);
    pub const NetworkError: Self = Self(1i32);
    pub const InvalidFormat: Self = Self(2i32);
    pub const Other: Self = Self(3i32);
}
impl ::core::marker::Copy for SvgImageSourceLoadStatus {}
impl ::core::clone::Clone for SvgImageSourceLoadStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SvgImageSourceOpenedEventArgs = *mut ::core::ffi::c_void;
pub type VirtualSurfaceImageSource = *mut ::core::ffi::c_void;
pub type WriteableBitmap = *mut ::core::ffi::c_void;
pub type XamlRenderingBackgroundTask = *mut ::core::ffi::c_void;
