#[doc = "*Required features: `\"Media_SpeechSynthesis\"`*"]
#[repr(transparent)]
pub struct SpeechAppendedSilence(pub i32);
impl SpeechAppendedSilence {
    pub const Default: Self = Self(0i32);
    pub const Min: Self = Self(1i32);
}
impl ::core::marker::Copy for SpeechAppendedSilence {}
impl ::core::clone::Clone for SpeechAppendedSilence {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_SpeechSynthesis\"`*"]
#[repr(transparent)]
pub struct SpeechPunctuationSilence(pub i32);
impl SpeechPunctuationSilence {
    pub const Default: Self = Self(0i32);
    pub const Min: Self = Self(1i32);
}
impl ::core::marker::Copy for SpeechPunctuationSilence {}
impl ::core::clone::Clone for SpeechPunctuationSilence {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpeechSynthesisStream = *mut ::core::ffi::c_void;
pub type SpeechSynthesizer = *mut ::core::ffi::c_void;
pub type SpeechSynthesizerOptions = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_SpeechSynthesis\"`*"]
#[repr(transparent)]
pub struct VoiceGender(pub i32);
impl VoiceGender {
    pub const Male: Self = Self(0i32);
    pub const Female: Self = Self(1i32);
}
impl ::core::marker::Copy for VoiceGender {}
impl ::core::clone::Clone for VoiceGender {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VoiceInformation = *mut ::core::ffi::c_void;
