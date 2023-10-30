#[cfg(feature = "Media_Devices_Core")]
pub mod Core;
pub type AdvancedPhotoCaptureSettings = *mut ::core::ffi::c_void;
pub type AdvancedPhotoControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct AdvancedPhotoMode(pub i32);
impl AdvancedPhotoMode {
    pub const Auto: Self = Self(0i32);
    pub const Standard: Self = Self(1i32);
    pub const Hdr: Self = Self(2i32);
    pub const LowLight: Self = Self(3i32);
}
impl ::core::marker::Copy for AdvancedPhotoMode {}
impl ::core::clone::Clone for AdvancedPhotoMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AudioDeviceController = *mut ::core::ffi::c_void;
pub type AudioDeviceModule = *mut ::core::ffi::c_void;
pub type AudioDeviceModuleNotificationEventArgs = *mut ::core::ffi::c_void;
pub type AudioDeviceModulesManager = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct AudioDeviceRole(pub i32);
impl AudioDeviceRole {
    pub const Default: Self = Self(0i32);
    pub const Communications: Self = Self(1i32);
}
impl ::core::marker::Copy for AudioDeviceRole {}
impl ::core::clone::Clone for AudioDeviceRole {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct AutoFocusRange(pub i32);
impl AutoFocusRange {
    pub const FullRange: Self = Self(0i32);
    pub const Macro: Self = Self(1i32);
    pub const Normal: Self = Self(2i32);
}
impl ::core::marker::Copy for AutoFocusRange {}
impl ::core::clone::Clone for AutoFocusRange {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CallControl = *mut ::core::ffi::c_void;
pub type CallControlEventHandler = *mut ::core::ffi::c_void;
pub type CameraOcclusionInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct CameraOcclusionKind(pub i32);
impl CameraOcclusionKind {
    pub const Lid: Self = Self(0i32);
    pub const CameraHardware: Self = Self(1i32);
}
impl ::core::marker::Copy for CameraOcclusionKind {}
impl ::core::clone::Clone for CameraOcclusionKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CameraOcclusionState = *mut ::core::ffi::c_void;
pub type CameraOcclusionStateChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct CameraStreamState(pub i32);
impl CameraStreamState {
    pub const NotStreaming: Self = Self(0i32);
    pub const Streaming: Self = Self(1i32);
    pub const BlockedForPrivacy: Self = Self(2i32);
    pub const Shutdown: Self = Self(3i32);
}
impl ::core::marker::Copy for CameraStreamState {}
impl ::core::clone::Clone for CameraStreamState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct CaptureSceneMode(pub i32);
impl CaptureSceneMode {
    pub const Auto: Self = Self(0i32);
    pub const Manual: Self = Self(1i32);
    pub const Macro: Self = Self(2i32);
    pub const Portrait: Self = Self(3i32);
    pub const Sport: Self = Self(4i32);
    pub const Snow: Self = Self(5i32);
    pub const Night: Self = Self(6i32);
    pub const Beach: Self = Self(7i32);
    pub const Sunset: Self = Self(8i32);
    pub const Candlelight: Self = Self(9i32);
    pub const Landscape: Self = Self(10i32);
    pub const NightPortrait: Self = Self(11i32);
    pub const Backlit: Self = Self(12i32);
}
impl ::core::marker::Copy for CaptureSceneMode {}
impl ::core::clone::Clone for CaptureSceneMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct CaptureUse(pub i32);
impl CaptureUse {
    pub const None: Self = Self(0i32);
    pub const Photo: Self = Self(1i32);
    pub const Video: Self = Self(2i32);
}
impl ::core::marker::Copy for CaptureUse {}
impl ::core::clone::Clone for CaptureUse {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct ColorTemperaturePreset(pub i32);
impl ColorTemperaturePreset {
    pub const Auto: Self = Self(0i32);
    pub const Manual: Self = Self(1i32);
    pub const Cloudy: Self = Self(2i32);
    pub const Daylight: Self = Self(3i32);
    pub const Flash: Self = Self(4i32);
    pub const Fluorescent: Self = Self(5i32);
    pub const Tungsten: Self = Self(6i32);
    pub const Candlelight: Self = Self(7i32);
}
impl ::core::marker::Copy for ColorTemperaturePreset {}
impl ::core::clone::Clone for ColorTemperaturePreset {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DefaultAudioCaptureDeviceChangedEventArgs = *mut ::core::ffi::c_void;
pub type DefaultAudioRenderDeviceChangedEventArgs = *mut ::core::ffi::c_void;
pub type DialRequestedEventArgs = *mut ::core::ffi::c_void;
pub type DialRequestedEventHandler = *mut ::core::ffi::c_void;
pub type DigitalWindowBounds = *mut ::core::ffi::c_void;
pub type DigitalWindowCapability = *mut ::core::ffi::c_void;
pub type DigitalWindowControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct DigitalWindowMode(pub i32);
impl DigitalWindowMode {
    pub const Off: Self = Self(0i32);
    pub const On: Self = Self(1i32);
    pub const Auto: Self = Self(2i32);
}
impl ::core::marker::Copy for DigitalWindowMode {}
impl ::core::clone::Clone for DigitalWindowMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ExposureCompensationControl = *mut ::core::ffi::c_void;
pub type ExposureControl = *mut ::core::ffi::c_void;
pub type ExposurePriorityVideoControl = *mut ::core::ffi::c_void;
pub type FlashControl = *mut ::core::ffi::c_void;
pub type FocusControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct FocusMode(pub i32);
impl FocusMode {
    pub const Auto: Self = Self(0i32);
    pub const Single: Self = Self(1i32);
    pub const Continuous: Self = Self(2i32);
    pub const Manual: Self = Self(3i32);
}
impl ::core::marker::Copy for FocusMode {}
impl ::core::clone::Clone for FocusMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct FocusPreset(pub i32);
impl FocusPreset {
    pub const Auto: Self = Self(0i32);
    pub const Manual: Self = Self(1i32);
    pub const AutoMacro: Self = Self(2i32);
    pub const AutoNormal: Self = Self(3i32);
    pub const AutoInfinity: Self = Self(4i32);
    pub const AutoHyperfocal: Self = Self(5i32);
}
impl ::core::marker::Copy for FocusPreset {}
impl ::core::clone::Clone for FocusPreset {
    fn clone(&self) -> Self {
        *self
    }
}
pub type FocusSettings = *mut ::core::ffi::c_void;
pub type HdrVideoControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct HdrVideoMode(pub i32);
impl HdrVideoMode {
    pub const Off: Self = Self(0i32);
    pub const On: Self = Self(1i32);
    pub const Auto: Self = Self(2i32);
}
impl ::core::marker::Copy for HdrVideoMode {}
impl ::core::clone::Clone for HdrVideoMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IDefaultAudioDeviceChangedEventArgs = *mut ::core::ffi::c_void;
pub type IMediaDeviceController = *mut ::core::ffi::c_void;
pub type InfraredTorchControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct InfraredTorchMode(pub i32);
impl InfraredTorchMode {
    pub const Off: Self = Self(0i32);
    pub const On: Self = Self(1i32);
    pub const AlternatingFrameIllumination: Self = Self(2i32);
}
impl ::core::marker::Copy for InfraredTorchMode {}
impl ::core::clone::Clone for InfraredTorchMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IsoSpeedControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct IsoSpeedPreset(pub i32);
#[cfg(feature = "deprecated")]
impl IsoSpeedPreset {
    pub const Auto: Self = Self(0i32);
    pub const Iso50: Self = Self(1i32);
    pub const Iso80: Self = Self(2i32);
    pub const Iso100: Self = Self(3i32);
    pub const Iso200: Self = Self(4i32);
    pub const Iso400: Self = Self(5i32);
    pub const Iso800: Self = Self(6i32);
    pub const Iso1600: Self = Self(7i32);
    pub const Iso3200: Self = Self(8i32);
    pub const Iso6400: Self = Self(9i32);
    pub const Iso12800: Self = Self(10i32);
    pub const Iso25600: Self = Self(11i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for IsoSpeedPreset {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for IsoSpeedPreset {
    fn clone(&self) -> Self {
        *self
    }
}
pub type KeypadPressedEventArgs = *mut ::core::ffi::c_void;
pub type KeypadPressedEventHandler = *mut ::core::ffi::c_void;
pub type LowLagPhotoControl = *mut ::core::ffi::c_void;
pub type LowLagPhotoSequenceControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct ManualFocusDistance(pub i32);
impl ManualFocusDistance {
    pub const Infinity: Self = Self(0i32);
    pub const Hyperfocal: Self = Self(1i32);
    pub const Nearest: Self = Self(2i32);
}
impl ::core::marker::Copy for ManualFocusDistance {}
impl ::core::clone::Clone for ManualFocusDistance {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct MediaCaptureFocusState(pub i32);
impl MediaCaptureFocusState {
    pub const Uninitialized: Self = Self(0i32);
    pub const Lost: Self = Self(1i32);
    pub const Searching: Self = Self(2i32);
    pub const Focused: Self = Self(3i32);
    pub const Failed: Self = Self(4i32);
}
impl ::core::marker::Copy for MediaCaptureFocusState {}
impl ::core::clone::Clone for MediaCaptureFocusState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct MediaCaptureOptimization(pub i32);
impl MediaCaptureOptimization {
    pub const Default: Self = Self(0i32);
    pub const Quality: Self = Self(1i32);
    pub const Latency: Self = Self(2i32);
    pub const Power: Self = Self(3i32);
    pub const LatencyThenQuality: Self = Self(4i32);
    pub const LatencyThenPower: Self = Self(5i32);
    pub const PowerAndQuality: Self = Self(6i32);
}
impl ::core::marker::Copy for MediaCaptureOptimization {}
impl ::core::clone::Clone for MediaCaptureOptimization {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct MediaCapturePauseBehavior(pub i32);
impl MediaCapturePauseBehavior {
    pub const RetainHardwareResources: Self = Self(0i32);
    pub const ReleaseHardwareResources: Self = Self(1i32);
}
impl ::core::marker::Copy for MediaCapturePauseBehavior {}
impl ::core::clone::Clone for MediaCapturePauseBehavior {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaDeviceControl = *mut ::core::ffi::c_void;
pub type MediaDeviceControlCapabilities = *mut ::core::ffi::c_void;
pub type ModuleCommandResult = *mut ::core::ffi::c_void;
pub type OpticalImageStabilizationControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct OpticalImageStabilizationMode(pub i32);
impl OpticalImageStabilizationMode {
    pub const Off: Self = Self(0i32);
    pub const On: Self = Self(1i32);
    pub const Auto: Self = Self(2i32);
}
impl ::core::marker::Copy for OpticalImageStabilizationMode {}
impl ::core::clone::Clone for OpticalImageStabilizationMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PanelBasedOptimizationControl = *mut ::core::ffi::c_void;
pub type PhotoConfirmationControl = *mut ::core::ffi::c_void;
pub type RedialRequestedEventArgs = *mut ::core::ffi::c_void;
pub type RedialRequestedEventHandler = *mut ::core::ffi::c_void;
pub type RegionOfInterest = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct RegionOfInterestType(pub i32);
impl RegionOfInterestType {
    pub const Unknown: Self = Self(0i32);
    pub const Face: Self = Self(1i32);
}
impl ::core::marker::Copy for RegionOfInterestType {}
impl ::core::clone::Clone for RegionOfInterestType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RegionsOfInterestControl = *mut ::core::ffi::c_void;
pub type SceneModeControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct SendCommandStatus(pub i32);
impl SendCommandStatus {
    pub const Success: Self = Self(0i32);
    pub const DeviceNotAvailable: Self = Self(1i32);
}
impl ::core::marker::Copy for SendCommandStatus {}
impl ::core::clone::Clone for SendCommandStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct TelephonyKey(pub i32);
impl TelephonyKey {
    pub const D0: Self = Self(0i32);
    pub const D1: Self = Self(1i32);
    pub const D2: Self = Self(2i32);
    pub const D3: Self = Self(3i32);
    pub const D4: Self = Self(4i32);
    pub const D5: Self = Self(5i32);
    pub const D6: Self = Self(6i32);
    pub const D7: Self = Self(7i32);
    pub const D8: Self = Self(8i32);
    pub const D9: Self = Self(9i32);
    pub const Star: Self = Self(10i32);
    pub const Pound: Self = Self(11i32);
    pub const A: Self = Self(12i32);
    pub const B: Self = Self(13i32);
    pub const C: Self = Self(14i32);
    pub const D: Self = Self(15i32);
}
impl ::core::marker::Copy for TelephonyKey {}
impl ::core::clone::Clone for TelephonyKey {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TorchControl = *mut ::core::ffi::c_void;
pub type VideoDeviceController = *mut ::core::ffi::c_void;
pub type VideoDeviceControllerGetDevicePropertyResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct VideoDeviceControllerGetDevicePropertyStatus(pub i32);
impl VideoDeviceControllerGetDevicePropertyStatus {
    pub const Success: Self = Self(0i32);
    pub const UnknownFailure: Self = Self(1i32);
    pub const BufferTooSmall: Self = Self(2i32);
    pub const NotSupported: Self = Self(3i32);
    pub const DeviceNotAvailable: Self = Self(4i32);
    pub const MaxPropertyValueSizeTooSmall: Self = Self(5i32);
    pub const MaxPropertyValueSizeRequired: Self = Self(6i32);
}
impl ::core::marker::Copy for VideoDeviceControllerGetDevicePropertyStatus {}
impl ::core::clone::Clone for VideoDeviceControllerGetDevicePropertyStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct VideoDeviceControllerSetDevicePropertyStatus(pub i32);
impl VideoDeviceControllerSetDevicePropertyStatus {
    pub const Success: Self = Self(0i32);
    pub const UnknownFailure: Self = Self(1i32);
    pub const NotSupported: Self = Self(2i32);
    pub const InvalidValue: Self = Self(3i32);
    pub const DeviceNotAvailable: Self = Self(4i32);
    pub const NotInControl: Self = Self(5i32);
}
impl ::core::marker::Copy for VideoDeviceControllerSetDevicePropertyStatus {}
impl ::core::clone::Clone for VideoDeviceControllerSetDevicePropertyStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type VideoTemporalDenoisingControl = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct VideoTemporalDenoisingMode(pub i32);
impl VideoTemporalDenoisingMode {
    pub const Off: Self = Self(0i32);
    pub const On: Self = Self(1i32);
    pub const Auto: Self = Self(2i32);
}
impl ::core::marker::Copy for VideoTemporalDenoisingMode {}
impl ::core::clone::Clone for VideoTemporalDenoisingMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WhiteBalanceControl = *mut ::core::ffi::c_void;
pub type ZoomControl = *mut ::core::ffi::c_void;
pub type ZoomSettings = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Devices\"`*"]
#[repr(transparent)]
pub struct ZoomTransitionMode(pub i32);
impl ZoomTransitionMode {
    pub const Auto: Self = Self(0i32);
    pub const Direct: Self = Self(1i32);
    pub const Smooth: Self = Self(2i32);
}
impl ::core::marker::Copy for ZoomTransitionMode {}
impl ::core::clone::Clone for ZoomTransitionMode {
    fn clone(&self) -> Self {
        *self
    }
}
