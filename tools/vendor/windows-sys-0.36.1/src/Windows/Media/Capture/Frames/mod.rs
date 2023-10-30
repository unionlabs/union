pub type AudioMediaFrame = *mut ::core::ffi::c_void;
pub type BufferMediaFrame = *mut ::core::ffi::c_void;
pub type DepthMediaFrame = *mut ::core::ffi::c_void;
pub type DepthMediaFrameFormat = *mut ::core::ffi::c_void;
pub type InfraredMediaFrame = *mut ::core::ffi::c_void;
pub type MediaFrameArrivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaFrameFormat = *mut ::core::ffi::c_void;
pub type MediaFrameReader = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Capture_Frames\"`*"]
#[repr(transparent)]
pub struct MediaFrameReaderAcquisitionMode(pub i32);
impl MediaFrameReaderAcquisitionMode {
    pub const Realtime: Self = Self(0i32);
    pub const Buffered: Self = Self(1i32);
}
impl ::core::marker::Copy for MediaFrameReaderAcquisitionMode {}
impl ::core::clone::Clone for MediaFrameReaderAcquisitionMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Capture_Frames\"`*"]
#[repr(transparent)]
pub struct MediaFrameReaderStartStatus(pub i32);
impl MediaFrameReaderStartStatus {
    pub const Success: Self = Self(0i32);
    pub const UnknownFailure: Self = Self(1i32);
    pub const DeviceNotAvailable: Self = Self(2i32);
    pub const OutputFormatNotSupported: Self = Self(3i32);
    pub const ExclusiveControlNotAvailable: Self = Self(4i32);
}
impl ::core::marker::Copy for MediaFrameReaderStartStatus {}
impl ::core::clone::Clone for MediaFrameReaderStartStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaFrameReference = *mut ::core::ffi::c_void;
pub type MediaFrameSource = *mut ::core::ffi::c_void;
pub type MediaFrameSourceController = *mut ::core::ffi::c_void;
pub type MediaFrameSourceGetPropertyResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Capture_Frames\"`*"]
#[repr(transparent)]
pub struct MediaFrameSourceGetPropertyStatus(pub i32);
impl MediaFrameSourceGetPropertyStatus {
    pub const Success: Self = Self(0i32);
    pub const UnknownFailure: Self = Self(1i32);
    pub const NotSupported: Self = Self(2i32);
    pub const DeviceNotAvailable: Self = Self(3i32);
    pub const MaxPropertyValueSizeTooSmall: Self = Self(4i32);
    pub const MaxPropertyValueSizeRequired: Self = Self(5i32);
}
impl ::core::marker::Copy for MediaFrameSourceGetPropertyStatus {}
impl ::core::clone::Clone for MediaFrameSourceGetPropertyStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaFrameSourceGroup = *mut ::core::ffi::c_void;
pub type MediaFrameSourceInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Capture_Frames\"`*"]
#[repr(transparent)]
pub struct MediaFrameSourceKind(pub i32);
impl MediaFrameSourceKind {
    pub const Custom: Self = Self(0i32);
    pub const Color: Self = Self(1i32);
    pub const Infrared: Self = Self(2i32);
    pub const Depth: Self = Self(3i32);
    pub const Audio: Self = Self(4i32);
    pub const Image: Self = Self(5i32);
    pub const Metadata: Self = Self(6i32);
}
impl ::core::marker::Copy for MediaFrameSourceKind {}
impl ::core::clone::Clone for MediaFrameSourceKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Capture_Frames\"`*"]
#[repr(transparent)]
pub struct MediaFrameSourceSetPropertyStatus(pub i32);
impl MediaFrameSourceSetPropertyStatus {
    pub const Success: Self = Self(0i32);
    pub const UnknownFailure: Self = Self(1i32);
    pub const NotSupported: Self = Self(2i32);
    pub const InvalidValue: Self = Self(3i32);
    pub const DeviceNotAvailable: Self = Self(4i32);
    pub const NotInControl: Self = Self(5i32);
}
impl ::core::marker::Copy for MediaFrameSourceSetPropertyStatus {}
impl ::core::clone::Clone for MediaFrameSourceSetPropertyStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MultiSourceMediaFrameArrivedEventArgs = *mut ::core::ffi::c_void;
pub type MultiSourceMediaFrameReader = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Capture_Frames\"`*"]
#[repr(transparent)]
pub struct MultiSourceMediaFrameReaderStartStatus(pub i32);
impl MultiSourceMediaFrameReaderStartStatus {
    pub const Success: Self = Self(0i32);
    pub const NotSupported: Self = Self(1i32);
    pub const InsufficientResources: Self = Self(2i32);
    pub const DeviceNotAvailable: Self = Self(3i32);
    pub const UnknownFailure: Self = Self(4i32);
}
impl ::core::marker::Copy for MultiSourceMediaFrameReaderStartStatus {}
impl ::core::clone::Clone for MultiSourceMediaFrameReaderStartStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MultiSourceMediaFrameReference = *mut ::core::ffi::c_void;
pub type VideoMediaFrame = *mut ::core::ffi::c_void;
pub type VideoMediaFrameFormat = *mut ::core::ffi::c_void;
