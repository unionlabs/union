pub type MediaTranscoder = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Transcoding\"`*"]
#[repr(transparent)]
pub struct MediaVideoProcessingAlgorithm(pub i32);
impl MediaVideoProcessingAlgorithm {
    pub const Default: Self = Self(0i32);
    pub const MrfCrf444: Self = Self(1i32);
}
impl ::core::marker::Copy for MediaVideoProcessingAlgorithm {}
impl ::core::clone::Clone for MediaVideoProcessingAlgorithm {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrepareTranscodeResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Transcoding\"`*"]
#[repr(transparent)]
pub struct TranscodeFailureReason(pub i32);
impl TranscodeFailureReason {
    pub const None: Self = Self(0i32);
    pub const Unknown: Self = Self(1i32);
    pub const InvalidProfile: Self = Self(2i32);
    pub const CodecNotFound: Self = Self(3i32);
}
impl ::core::marker::Copy for TranscodeFailureReason {}
impl ::core::clone::Clone for TranscodeFailureReason {
    fn clone(&self) -> Self {
        *self
    }
}
