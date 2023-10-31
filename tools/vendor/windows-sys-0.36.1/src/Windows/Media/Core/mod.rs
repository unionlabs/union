#[cfg(feature = "Media_Core_Preview")]
pub mod Preview;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct AudioDecoderDegradation(pub i32);
impl AudioDecoderDegradation {
    pub const None: Self = Self(0i32);
    pub const DownmixTo2Channels: Self = Self(1i32);
    pub const DownmixTo6Channels: Self = Self(2i32);
    pub const DownmixTo8Channels: Self = Self(3i32);
}
impl ::core::marker::Copy for AudioDecoderDegradation {}
impl ::core::clone::Clone for AudioDecoderDegradation {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct AudioDecoderDegradationReason(pub i32);
impl AudioDecoderDegradationReason {
    pub const None: Self = Self(0i32);
    pub const LicensingRequirement: Self = Self(1i32);
    pub const SpatialAudioNotSupported: Self = Self(2i32);
}
impl ::core::marker::Copy for AudioDecoderDegradationReason {}
impl ::core::clone::Clone for AudioDecoderDegradationReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AudioStreamDescriptor = *mut ::core::ffi::c_void;
pub type AudioTrack = *mut ::core::ffi::c_void;
pub type AudioTrackOpenFailedEventArgs = *mut ::core::ffi::c_void;
pub type AudioTrackSupportInfo = *mut ::core::ffi::c_void;
pub type ChapterCue = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct CodecCategory(pub i32);
impl CodecCategory {
    pub const Encoder: Self = Self(0i32);
    pub const Decoder: Self = Self(1i32);
}
impl ::core::marker::Copy for CodecCategory {}
impl ::core::clone::Clone for CodecCategory {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CodecInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct CodecKind(pub i32);
impl CodecKind {
    pub const Audio: Self = Self(0i32);
    pub const Video: Self = Self(1i32);
}
impl ::core::marker::Copy for CodecKind {}
impl ::core::clone::Clone for CodecKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CodecQuery = *mut ::core::ffi::c_void;
pub type DataCue = *mut ::core::ffi::c_void;
pub type FaceDetectedEventArgs = *mut ::core::ffi::c_void;
pub type FaceDetectionEffect = *mut ::core::ffi::c_void;
pub type FaceDetectionEffectDefinition = *mut ::core::ffi::c_void;
pub type FaceDetectionEffectFrame = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct FaceDetectionMode(pub i32);
impl FaceDetectionMode {
    pub const HighPerformance: Self = Self(0i32);
    pub const Balanced: Self = Self(1i32);
    pub const HighQuality: Self = Self(2i32);
}
impl ::core::marker::Copy for FaceDetectionMode {}
impl ::core::clone::Clone for FaceDetectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HighDynamicRangeControl = *mut ::core::ffi::c_void;
pub type HighDynamicRangeOutput = *mut ::core::ffi::c_void;
pub type IMediaCue = *mut ::core::ffi::c_void;
pub type IMediaSource = *mut ::core::ffi::c_void;
pub type IMediaStreamDescriptor = *mut ::core::ffi::c_void;
pub type IMediaStreamDescriptor2 = *mut ::core::ffi::c_void;
pub type IMediaTrack = *mut ::core::ffi::c_void;
pub type ISingleSelectMediaTrackList = *mut ::core::ffi::c_void;
pub type ITimedMetadataTrackProvider = *mut ::core::ffi::c_void;
pub type ImageCue = *mut ::core::ffi::c_void;
pub type InitializeMediaStreamSourceRequestedEventArgs = *mut ::core::ffi::c_void;
pub type LowLightFusionResult = *mut ::core::ffi::c_void;
pub type MediaBinder = *mut ::core::ffi::c_void;
pub type MediaBindingEventArgs = *mut ::core::ffi::c_void;
pub type MediaCueEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MediaDecoderStatus(pub i32);
impl MediaDecoderStatus {
    pub const FullySupported: Self = Self(0i32);
    pub const UnsupportedSubtype: Self = Self(1i32);
    pub const UnsupportedEncoderProperties: Self = Self(2i32);
    pub const Degraded: Self = Self(3i32);
}
impl ::core::marker::Copy for MediaDecoderStatus {}
impl ::core::clone::Clone for MediaDecoderStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaSource = *mut ::core::ffi::c_void;
pub type MediaSourceAppServiceConnection = *mut ::core::ffi::c_void;
pub type MediaSourceError = *mut ::core::ffi::c_void;
pub type MediaSourceOpenOperationCompletedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MediaSourceState(pub i32);
impl MediaSourceState {
    pub const Initial: Self = Self(0i32);
    pub const Opening: Self = Self(1i32);
    pub const Opened: Self = Self(2i32);
    pub const Failed: Self = Self(3i32);
    pub const Closed: Self = Self(4i32);
}
impl ::core::marker::Copy for MediaSourceState {}
impl ::core::clone::Clone for MediaSourceState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaSourceStateChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MediaSourceStatus(pub i32);
impl MediaSourceStatus {
    pub const FullySupported: Self = Self(0i32);
    pub const Unknown: Self = Self(1i32);
}
impl ::core::marker::Copy for MediaSourceStatus {}
impl ::core::clone::Clone for MediaSourceStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaStreamSample = *mut ::core::ffi::c_void;
pub type MediaStreamSamplePropertySet = *mut ::core::ffi::c_void;
pub type MediaStreamSampleProtectionProperties = *mut ::core::ffi::c_void;
pub type MediaStreamSource = *mut ::core::ffi::c_void;
pub type MediaStreamSourceClosedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MediaStreamSourceClosedReason(pub i32);
impl MediaStreamSourceClosedReason {
    pub const Done: Self = Self(0i32);
    pub const UnknownError: Self = Self(1i32);
    pub const AppReportedError: Self = Self(2i32);
    pub const UnsupportedProtectionSystem: Self = Self(3i32);
    pub const ProtectionSystemFailure: Self = Self(4i32);
    pub const UnsupportedEncodingFormat: Self = Self(5i32);
    pub const MissingSampleRequestedEventHandler: Self = Self(6i32);
}
impl ::core::marker::Copy for MediaStreamSourceClosedReason {}
impl ::core::clone::Clone for MediaStreamSourceClosedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaStreamSourceClosedRequest = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MediaStreamSourceErrorStatus(pub i32);
impl MediaStreamSourceErrorStatus {
    pub const Other: Self = Self(0i32);
    pub const OutOfMemory: Self = Self(1i32);
    pub const FailedToOpenFile: Self = Self(2i32);
    pub const FailedToConnectToServer: Self = Self(3i32);
    pub const ConnectionToServerLost: Self = Self(4i32);
    pub const UnspecifiedNetworkError: Self = Self(5i32);
    pub const DecodeError: Self = Self(6i32);
    pub const UnsupportedMediaFormat: Self = Self(7i32);
}
impl ::core::marker::Copy for MediaStreamSourceErrorStatus {}
impl ::core::clone::Clone for MediaStreamSourceErrorStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaStreamSourceSampleRenderedEventArgs = *mut ::core::ffi::c_void;
pub type MediaStreamSourceSampleRequest = *mut ::core::ffi::c_void;
pub type MediaStreamSourceSampleRequestDeferral = *mut ::core::ffi::c_void;
pub type MediaStreamSourceSampleRequestedEventArgs = *mut ::core::ffi::c_void;
pub type MediaStreamSourceStartingEventArgs = *mut ::core::ffi::c_void;
pub type MediaStreamSourceStartingRequest = *mut ::core::ffi::c_void;
pub type MediaStreamSourceStartingRequestDeferral = *mut ::core::ffi::c_void;
pub type MediaStreamSourceSwitchStreamsRequest = *mut ::core::ffi::c_void;
pub type MediaStreamSourceSwitchStreamsRequestDeferral = *mut ::core::ffi::c_void;
pub type MediaStreamSourceSwitchStreamsRequestedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MediaTrackKind(pub i32);
impl MediaTrackKind {
    pub const Audio: Self = Self(0i32);
    pub const Video: Self = Self(1i32);
    pub const TimedMetadata: Self = Self(2i32);
}
impl ::core::marker::Copy for MediaTrackKind {}
impl ::core::clone::Clone for MediaTrackKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MseAppendMode(pub i32);
impl MseAppendMode {
    pub const Segments: Self = Self(0i32);
    pub const Sequence: Self = Self(1i32);
}
impl ::core::marker::Copy for MseAppendMode {}
impl ::core::clone::Clone for MseAppendMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MseEndOfStreamStatus(pub i32);
impl MseEndOfStreamStatus {
    pub const Success: Self = Self(0i32);
    pub const NetworkError: Self = Self(1i32);
    pub const DecodeError: Self = Self(2i32);
    pub const UnknownError: Self = Self(3i32);
}
impl ::core::marker::Copy for MseEndOfStreamStatus {}
impl ::core::clone::Clone for MseEndOfStreamStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct MseReadyState(pub i32);
impl MseReadyState {
    pub const Closed: Self = Self(0i32);
    pub const Open: Self = Self(1i32);
    pub const Ended: Self = Self(2i32);
}
impl ::core::marker::Copy for MseReadyState {}
impl ::core::clone::Clone for MseReadyState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MseSourceBuffer = *mut ::core::ffi::c_void;
pub type MseSourceBufferList = *mut ::core::ffi::c_void;
pub type MseStreamSource = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Media_Core\"`, `\"Foundation\"`*"]
#[cfg(feature = "Foundation")]
pub struct MseTimeRange {
    pub Start: super::super::Foundation::TimeSpan,
    pub End: super::super::Foundation::TimeSpan,
}
#[cfg(feature = "Foundation")]
impl ::core::marker::Copy for MseTimeRange {}
#[cfg(feature = "Foundation")]
impl ::core::clone::Clone for MseTimeRange {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SceneAnalysisEffect = *mut ::core::ffi::c_void;
pub type SceneAnalysisEffectDefinition = *mut ::core::ffi::c_void;
pub type SceneAnalysisEffectFrame = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct SceneAnalysisRecommendation(pub i32);
impl SceneAnalysisRecommendation {
    pub const Standard: Self = Self(0i32);
    pub const Hdr: Self = Self(1i32);
    pub const LowLight: Self = Self(2i32);
}
impl ::core::marker::Copy for SceneAnalysisRecommendation {}
impl ::core::clone::Clone for SceneAnalysisRecommendation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SceneAnalyzedEventArgs = *mut ::core::ffi::c_void;
pub type SpeechCue = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedMetadataKind(pub i32);
impl TimedMetadataKind {
    pub const Caption: Self = Self(0i32);
    pub const Chapter: Self = Self(1i32);
    pub const Custom: Self = Self(2i32);
    pub const Data: Self = Self(3i32);
    pub const Description: Self = Self(4i32);
    pub const Subtitle: Self = Self(5i32);
    pub const ImageSubtitle: Self = Self(6i32);
    pub const Speech: Self = Self(7i32);
}
impl ::core::marker::Copy for TimedMetadataKind {}
impl ::core::clone::Clone for TimedMetadataKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TimedMetadataStreamDescriptor = *mut ::core::ffi::c_void;
pub type TimedMetadataTrack = *mut ::core::ffi::c_void;
pub type TimedMetadataTrackError = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedMetadataTrackErrorCode(pub i32);
impl TimedMetadataTrackErrorCode {
    pub const None: Self = Self(0i32);
    pub const DataFormatError: Self = Self(1i32);
    pub const NetworkError: Self = Self(2i32);
    pub const InternalError: Self = Self(3i32);
}
impl ::core::marker::Copy for TimedMetadataTrackErrorCode {}
impl ::core::clone::Clone for TimedMetadataTrackErrorCode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TimedMetadataTrackFailedEventArgs = *mut ::core::ffi::c_void;
pub type TimedTextBouten = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextBoutenPosition(pub i32);
impl TimedTextBoutenPosition {
    pub const Before: Self = Self(0i32);
    pub const After: Self = Self(1i32);
    pub const Outside: Self = Self(2i32);
}
impl ::core::marker::Copy for TimedTextBoutenPosition {}
impl ::core::clone::Clone for TimedTextBoutenPosition {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextBoutenType(pub i32);
impl TimedTextBoutenType {
    pub const None: Self = Self(0i32);
    pub const Auto: Self = Self(1i32);
    pub const FilledCircle: Self = Self(2i32);
    pub const OpenCircle: Self = Self(3i32);
    pub const FilledDot: Self = Self(4i32);
    pub const OpenDot: Self = Self(5i32);
    pub const FilledSesame: Self = Self(6i32);
    pub const OpenSesame: Self = Self(7i32);
}
impl ::core::marker::Copy for TimedTextBoutenType {}
impl ::core::clone::Clone for TimedTextBoutenType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TimedTextCue = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextDisplayAlignment(pub i32);
impl TimedTextDisplayAlignment {
    pub const Before: Self = Self(0i32);
    pub const After: Self = Self(1i32);
    pub const Center: Self = Self(2i32);
}
impl ::core::marker::Copy for TimedTextDisplayAlignment {}
impl ::core::clone::Clone for TimedTextDisplayAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Media_Core\"`*"]
pub struct TimedTextDouble {
    pub Value: f64,
    pub Unit: TimedTextUnit,
}
impl ::core::marker::Copy for TimedTextDouble {}
impl ::core::clone::Clone for TimedTextDouble {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextFlowDirection(pub i32);
impl TimedTextFlowDirection {
    pub const LeftToRight: Self = Self(0i32);
    pub const RightToLeft: Self = Self(1i32);
}
impl ::core::marker::Copy for TimedTextFlowDirection {}
impl ::core::clone::Clone for TimedTextFlowDirection {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextFontStyle(pub i32);
impl TimedTextFontStyle {
    pub const Normal: Self = Self(0i32);
    pub const Oblique: Self = Self(1i32);
    pub const Italic: Self = Self(2i32);
}
impl ::core::marker::Copy for TimedTextFontStyle {}
impl ::core::clone::Clone for TimedTextFontStyle {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TimedTextLine = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextLineAlignment(pub i32);
impl TimedTextLineAlignment {
    pub const Start: Self = Self(0i32);
    pub const End: Self = Self(1i32);
    pub const Center: Self = Self(2i32);
}
impl ::core::marker::Copy for TimedTextLineAlignment {}
impl ::core::clone::Clone for TimedTextLineAlignment {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Media_Core\"`*"]
pub struct TimedTextPadding {
    pub Before: f64,
    pub After: f64,
    pub Start: f64,
    pub End: f64,
    pub Unit: TimedTextUnit,
}
impl ::core::marker::Copy for TimedTextPadding {}
impl ::core::clone::Clone for TimedTextPadding {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Media_Core\"`*"]
pub struct TimedTextPoint {
    pub X: f64,
    pub Y: f64,
    pub Unit: TimedTextUnit,
}
impl ::core::marker::Copy for TimedTextPoint {}
impl ::core::clone::Clone for TimedTextPoint {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TimedTextRegion = *mut ::core::ffi::c_void;
pub type TimedTextRuby = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextRubyAlign(pub i32);
impl TimedTextRubyAlign {
    pub const Center: Self = Self(0i32);
    pub const Start: Self = Self(1i32);
    pub const End: Self = Self(2i32);
    pub const SpaceAround: Self = Self(3i32);
    pub const SpaceBetween: Self = Self(4i32);
    pub const WithBase: Self = Self(5i32);
}
impl ::core::marker::Copy for TimedTextRubyAlign {}
impl ::core::clone::Clone for TimedTextRubyAlign {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextRubyPosition(pub i32);
impl TimedTextRubyPosition {
    pub const Before: Self = Self(0i32);
    pub const After: Self = Self(1i32);
    pub const Outside: Self = Self(2i32);
}
impl ::core::marker::Copy for TimedTextRubyPosition {}
impl ::core::clone::Clone for TimedTextRubyPosition {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextRubyReserve(pub i32);
impl TimedTextRubyReserve {
    pub const None: Self = Self(0i32);
    pub const Before: Self = Self(1i32);
    pub const After: Self = Self(2i32);
    pub const Both: Self = Self(3i32);
    pub const Outside: Self = Self(4i32);
}
impl ::core::marker::Copy for TimedTextRubyReserve {}
impl ::core::clone::Clone for TimedTextRubyReserve {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextScrollMode(pub i32);
impl TimedTextScrollMode {
    pub const Popon: Self = Self(0i32);
    pub const Rollup: Self = Self(1i32);
}
impl ::core::marker::Copy for TimedTextScrollMode {}
impl ::core::clone::Clone for TimedTextScrollMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Media_Core\"`*"]
pub struct TimedTextSize {
    pub Height: f64,
    pub Width: f64,
    pub Unit: TimedTextUnit,
}
impl ::core::marker::Copy for TimedTextSize {}
impl ::core::clone::Clone for TimedTextSize {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TimedTextSource = *mut ::core::ffi::c_void;
pub type TimedTextSourceResolveResultEventArgs = *mut ::core::ffi::c_void;
pub type TimedTextStyle = *mut ::core::ffi::c_void;
pub type TimedTextSubformat = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextUnit(pub i32);
impl TimedTextUnit {
    pub const Pixels: Self = Self(0i32);
    pub const Percentage: Self = Self(1i32);
}
impl ::core::marker::Copy for TimedTextUnit {}
impl ::core::clone::Clone for TimedTextUnit {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextWeight(pub i32);
impl TimedTextWeight {
    pub const Normal: Self = Self(400i32);
    pub const Bold: Self = Self(700i32);
}
impl ::core::marker::Copy for TimedTextWeight {}
impl ::core::clone::Clone for TimedTextWeight {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextWrapping(pub i32);
impl TimedTextWrapping {
    pub const NoWrap: Self = Self(0i32);
    pub const Wrap: Self = Self(1i32);
}
impl ::core::marker::Copy for TimedTextWrapping {}
impl ::core::clone::Clone for TimedTextWrapping {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct TimedTextWritingMode(pub i32);
impl TimedTextWritingMode {
    pub const LeftRightTopBottom: Self = Self(0i32);
    pub const RightLeftTopBottom: Self = Self(1i32);
    pub const TopBottomRightLeft: Self = Self(2i32);
    pub const TopBottomLeftRight: Self = Self(3i32);
    pub const LeftRight: Self = Self(4i32);
    pub const RightLeft: Self = Self(5i32);
    pub const TopBottom: Self = Self(6i32);
}
impl ::core::marker::Copy for TimedTextWritingMode {}
impl ::core::clone::Clone for TimedTextWritingMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VideoStabilizationEffect = *mut ::core::ffi::c_void;
pub type VideoStabilizationEffectDefinition = *mut ::core::ffi::c_void;
pub type VideoStabilizationEffectEnabledChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Core\"`*"]
#[repr(transparent)]
pub struct VideoStabilizationEffectEnabledChangedReason(pub i32);
impl VideoStabilizationEffectEnabledChangedReason {
    pub const Programmatic: Self = Self(0i32);
    pub const PixelRateTooHigh: Self = Self(1i32);
    pub const RunningSlowly: Self = Self(2i32);
}
impl ::core::marker::Copy for VideoStabilizationEffectEnabledChangedReason {}
impl ::core::clone::Clone for VideoStabilizationEffectEnabledChangedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VideoStreamDescriptor = *mut ::core::ffi::c_void;
pub type VideoTrack = *mut ::core::ffi::c_void;
pub type VideoTrackOpenFailedEventArgs = *mut ::core::ffi::c_void;
pub type VideoTrackSupportInfo = *mut ::core::ffi::c_void;
