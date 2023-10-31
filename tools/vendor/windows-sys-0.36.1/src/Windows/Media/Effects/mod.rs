pub type AudioCaptureEffectsManager = *mut ::core::ffi::c_void;
pub type AudioEffect = *mut ::core::ffi::c_void;
pub type AudioEffectDefinition = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Effects\"`*"]
#[repr(transparent)]
pub struct AudioEffectType(pub i32);
impl AudioEffectType {
    pub const Other: Self = Self(0i32);
    pub const AcousticEchoCancellation: Self = Self(1i32);
    pub const NoiseSuppression: Self = Self(2i32);
    pub const AutomaticGainControl: Self = Self(3i32);
    pub const BeamForming: Self = Self(4i32);
    pub const ConstantToneRemoval: Self = Self(5i32);
    pub const Equalizer: Self = Self(6i32);
    pub const LoudnessEqualizer: Self = Self(7i32);
    pub const BassBoost: Self = Self(8i32);
    pub const VirtualSurround: Self = Self(9i32);
    pub const VirtualHeadphones: Self = Self(10i32);
    pub const SpeakerFill: Self = Self(11i32);
    pub const RoomCorrection: Self = Self(12i32);
    pub const BassManagement: Self = Self(13i32);
    pub const EnvironmentalEffects: Self = Self(14i32);
    pub const SpeakerProtection: Self = Self(15i32);
    pub const SpeakerCompensation: Self = Self(16i32);
    pub const DynamicRangeCompression: Self = Self(17i32);
    pub const FarFieldBeamForming: Self = Self(18i32);
    pub const DeepNoiseSuppression: Self = Self(19i32);
}
impl ::core::marker::Copy for AudioEffectType {}
impl ::core::clone::Clone for AudioEffectType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AudioRenderEffectsManager = *mut ::core::ffi::c_void;
pub type CompositeVideoFrameContext = *mut ::core::ffi::c_void;
pub type IAudioEffectDefinition = *mut ::core::ffi::c_void;
pub type IBasicAudioEffect = *mut ::core::ffi::c_void;
pub type IBasicVideoEffect = *mut ::core::ffi::c_void;
pub type IVideoCompositor = *mut ::core::ffi::c_void;
pub type IVideoCompositorDefinition = *mut ::core::ffi::c_void;
pub type IVideoEffectDefinition = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Effects\"`*"]
#[repr(transparent)]
pub struct MediaEffectClosedReason(pub i32);
impl MediaEffectClosedReason {
    pub const Done: Self = Self(0i32);
    pub const UnknownError: Self = Self(1i32);
    pub const UnsupportedEncodingFormat: Self = Self(2i32);
    pub const EffectCurrentlyUnloaded: Self = Self(3i32);
}
impl ::core::marker::Copy for MediaEffectClosedReason {}
impl ::core::clone::Clone for MediaEffectClosedReason {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Effects\"`*"]
#[repr(transparent)]
pub struct MediaMemoryTypes(pub i32);
impl MediaMemoryTypes {
    pub const Gpu: Self = Self(0i32);
    pub const Cpu: Self = Self(1i32);
    pub const GpuAndCpu: Self = Self(2i32);
}
impl ::core::marker::Copy for MediaMemoryTypes {}
impl ::core::clone::Clone for MediaMemoryTypes {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ProcessAudioFrameContext = *mut ::core::ffi::c_void;
pub type ProcessVideoFrameContext = *mut ::core::ffi::c_void;
pub type SlowMotionEffectDefinition = *mut ::core::ffi::c_void;
pub type VideoCompositorDefinition = *mut ::core::ffi::c_void;
pub type VideoEffectDefinition = *mut ::core::ffi::c_void;
pub type VideoTransformEffectDefinition = *mut ::core::ffi::c_void;
pub type VideoTransformSphericalProjection = *mut ::core::ffi::c_void;
