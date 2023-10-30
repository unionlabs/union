pub type DisplayAdapter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayBitsPerChannel(pub u32);
impl DisplayBitsPerChannel {
    pub const None: Self = Self(0u32);
    pub const Bpc6: Self = Self(1u32);
    pub const Bpc8: Self = Self(2u32);
    pub const Bpc10: Self = Self(4u32);
    pub const Bpc12: Self = Self(8u32);
    pub const Bpc14: Self = Self(16u32);
    pub const Bpc16: Self = Self(32u32);
}
impl ::core::marker::Copy for DisplayBitsPerChannel {}
impl ::core::clone::Clone for DisplayBitsPerChannel {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayDevice = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayDeviceCapability(pub i32);
impl DisplayDeviceCapability {
    pub const FlipOverride: Self = Self(0i32);
}
impl ::core::marker::Copy for DisplayDeviceCapability {}
impl ::core::clone::Clone for DisplayDeviceCapability {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayFence = *mut ::core::ffi::c_void;
pub type DisplayManager = *mut ::core::ffi::c_void;
pub type DisplayManagerChangedEventArgs = *mut ::core::ffi::c_void;
pub type DisplayManagerDisabledEventArgs = *mut ::core::ffi::c_void;
pub type DisplayManagerEnabledEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayManagerOptions(pub u32);
impl DisplayManagerOptions {
    pub const None: Self = Self(0u32);
    pub const EnforceSourceOwnership: Self = Self(1u32);
    pub const VirtualRefreshRateAware: Self = Self(2u32);
}
impl ::core::marker::Copy for DisplayManagerOptions {}
impl ::core::clone::Clone for DisplayManagerOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayManagerPathsFailedOrInvalidatedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayManagerResult(pub i32);
impl DisplayManagerResult {
    pub const Success: Self = Self(0i32);
    pub const UnknownFailure: Self = Self(1i32);
    pub const TargetAccessDenied: Self = Self(2i32);
    pub const TargetStale: Self = Self(3i32);
    pub const RemoteSessionNotSupported: Self = Self(4i32);
}
impl ::core::marker::Copy for DisplayManagerResult {}
impl ::core::clone::Clone for DisplayManagerResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayManagerResultWithState = *mut ::core::ffi::c_void;
pub type DisplayModeInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayModeQueryOptions(pub u32);
impl DisplayModeQueryOptions {
    pub const None: Self = Self(0u32);
    pub const OnlyPreferredResolution: Self = Self(1u32);
}
impl ::core::marker::Copy for DisplayModeQueryOptions {}
impl ::core::clone::Clone for DisplayModeQueryOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayPath = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayPathScaling(pub i32);
impl DisplayPathScaling {
    pub const Identity: Self = Self(0i32);
    pub const Centered: Self = Self(1i32);
    pub const Stretched: Self = Self(2i32);
    pub const AspectRatioStretched: Self = Self(3i32);
    pub const Custom: Self = Self(4i32);
    pub const DriverPreferred: Self = Self(5i32);
}
impl ::core::marker::Copy for DisplayPathScaling {}
impl ::core::clone::Clone for DisplayPathScaling {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayPathStatus(pub i32);
impl DisplayPathStatus {
    pub const Unknown: Self = Self(0i32);
    pub const Succeeded: Self = Self(1i32);
    pub const Pending: Self = Self(2i32);
    pub const Failed: Self = Self(3i32);
    pub const FailedAsync: Self = Self(4i32);
    pub const InvalidatedAsync: Self = Self(5i32);
}
impl ::core::marker::Copy for DisplayPathStatus {}
impl ::core::clone::Clone for DisplayPathStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayPresentStatus(pub i32);
impl DisplayPresentStatus {
    pub const Success: Self = Self(0i32);
    pub const SourceStatusPreventedPresent: Self = Self(1i32);
    pub const ScanoutInvalid: Self = Self(2i32);
    pub const SourceInvalid: Self = Self(3i32);
    pub const DeviceInvalid: Self = Self(4i32);
    pub const UnknownFailure: Self = Self(5i32);
}
impl ::core::marker::Copy for DisplayPresentStatus {}
impl ::core::clone::Clone for DisplayPresentStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Devices_Display_Core\"`, `\"Foundation_Numerics\"`*"]
#[cfg(feature = "Foundation_Numerics")]
pub struct DisplayPresentationRate {
    pub VerticalSyncRate: super::super::super::Foundation::Numerics::Rational,
    pub VerticalSyncsPerPresentation: i32,
}
#[cfg(feature = "Foundation_Numerics")]
impl ::core::marker::Copy for DisplayPresentationRate {}
#[cfg(feature = "Foundation_Numerics")]
impl ::core::clone::Clone for DisplayPresentationRate {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayPrimaryDescription = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayRotation(pub i32);
impl DisplayRotation {
    pub const None: Self = Self(0i32);
    pub const Clockwise90Degrees: Self = Self(1i32);
    pub const Clockwise180Degrees: Self = Self(2i32);
    pub const Clockwise270Degrees: Self = Self(3i32);
}
impl ::core::marker::Copy for DisplayRotation {}
impl ::core::clone::Clone for DisplayRotation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayScanout = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayScanoutOptions(pub u32);
impl DisplayScanoutOptions {
    pub const None: Self = Self(0u32);
    pub const AllowTearing: Self = Self(2u32);
}
impl ::core::marker::Copy for DisplayScanoutOptions {}
impl ::core::clone::Clone for DisplayScanoutOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplaySource = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplaySourceStatus(pub i32);
impl DisplaySourceStatus {
    pub const Active: Self = Self(0i32);
    pub const PoweredOff: Self = Self(1i32);
    pub const Invalid: Self = Self(2i32);
    pub const OwnedByAnotherDevice: Self = Self(3i32);
    pub const Unowned: Self = Self(4i32);
}
impl ::core::marker::Copy for DisplaySourceStatus {}
impl ::core::clone::Clone for DisplaySourceStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayState = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayStateApplyOptions(pub u32);
impl DisplayStateApplyOptions {
    pub const None: Self = Self(0u32);
    pub const FailIfStateChanged: Self = Self(1u32);
    pub const ForceReapply: Self = Self(2u32);
    pub const ForceModeEnumeration: Self = Self(4u32);
}
impl ::core::marker::Copy for DisplayStateApplyOptions {}
impl ::core::clone::Clone for DisplayStateApplyOptions {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayStateFunctionalizeOptions(pub u32);
impl DisplayStateFunctionalizeOptions {
    pub const None: Self = Self(0u32);
    pub const FailIfStateChanged: Self = Self(1u32);
    pub const ValidateTopologyOnly: Self = Self(2u32);
}
impl ::core::marker::Copy for DisplayStateFunctionalizeOptions {}
impl ::core::clone::Clone for DisplayStateFunctionalizeOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayStateOperationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayStateOperationStatus(pub i32);
impl DisplayStateOperationStatus {
    pub const Success: Self = Self(0i32);
    pub const PartialFailure: Self = Self(1i32);
    pub const UnknownFailure: Self = Self(2i32);
    pub const TargetOwnershipLost: Self = Self(3i32);
    pub const SystemStateChanged: Self = Self(4i32);
    pub const TooManyPathsForAdapter: Self = Self(5i32);
    pub const ModesNotSupported: Self = Self(6i32);
    pub const RemoteSessionNotSupported: Self = Self(7i32);
}
impl ::core::marker::Copy for DisplayStateOperationStatus {}
impl ::core::clone::Clone for DisplayStateOperationStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplaySurface = *mut ::core::ffi::c_void;
pub type DisplayTarget = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayTargetPersistence(pub i32);
impl DisplayTargetPersistence {
    pub const None: Self = Self(0i32);
    pub const BootPersisted: Self = Self(1i32);
    pub const TemporaryPersisted: Self = Self(2i32);
    pub const PathPersisted: Self = Self(3i32);
}
impl ::core::marker::Copy for DisplayTargetPersistence {}
impl ::core::clone::Clone for DisplayTargetPersistence {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayTask = *mut ::core::ffi::c_void;
pub type DisplayTaskPool = *mut ::core::ffi::c_void;
pub type DisplayTaskResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayTaskSignalKind(pub i32);
impl DisplayTaskSignalKind {
    pub const OnPresentFlipAway: Self = Self(0i32);
    pub const OnPresentFlipTo: Self = Self(1i32);
}
impl ::core::marker::Copy for DisplayTaskSignalKind {}
impl ::core::clone::Clone for DisplayTaskSignalKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DisplayView = *mut ::core::ffi::c_void;
pub type DisplayWireFormat = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayWireFormatColorSpace(pub i32);
impl DisplayWireFormatColorSpace {
    pub const BT709: Self = Self(0i32);
    pub const BT2020: Self = Self(1i32);
    pub const ProfileDefinedWideColorGamut: Self = Self(2i32);
}
impl ::core::marker::Copy for DisplayWireFormatColorSpace {}
impl ::core::clone::Clone for DisplayWireFormatColorSpace {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayWireFormatEotf(pub i32);
impl DisplayWireFormatEotf {
    pub const Sdr: Self = Self(0i32);
    pub const HdrSmpte2084: Self = Self(1i32);
}
impl ::core::marker::Copy for DisplayWireFormatEotf {}
impl ::core::clone::Clone for DisplayWireFormatEotf {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayWireFormatHdrMetadata(pub i32);
impl DisplayWireFormatHdrMetadata {
    pub const None: Self = Self(0i32);
    pub const Hdr10: Self = Self(1i32);
    pub const Hdr10Plus: Self = Self(2i32);
    pub const DolbyVisionLowLatency: Self = Self(3i32);
}
impl ::core::marker::Copy for DisplayWireFormatHdrMetadata {}
impl ::core::clone::Clone for DisplayWireFormatHdrMetadata {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Display_Core\"`*"]
#[repr(transparent)]
pub struct DisplayWireFormatPixelEncoding(pub i32);
impl DisplayWireFormatPixelEncoding {
    pub const Rgb444: Self = Self(0i32);
    pub const Ycc444: Self = Self(1i32);
    pub const Ycc422: Self = Self(2i32);
    pub const Ycc420: Self = Self(3i32);
    pub const Intensity: Self = Self(4i32);
}
impl ::core::marker::Copy for DisplayWireFormatPixelEncoding {}
impl ::core::clone::Clone for DisplayWireFormatPixelEncoding {
    fn clone(&self) -> Self {
        *self
    }
}
