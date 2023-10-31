pub type AdaptiveMediaSource = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceAdvancedSettings = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceCorrelatedTimes = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceCreationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Streaming_Adaptive\"`*"]
#[repr(transparent)]
pub struct AdaptiveMediaSourceCreationStatus(pub i32);
impl AdaptiveMediaSourceCreationStatus {
    pub const Success: Self = Self(0i32);
    pub const ManifestDownloadFailure: Self = Self(1i32);
    pub const ManifestParseFailure: Self = Self(2i32);
    pub const UnsupportedManifestContentType: Self = Self(3i32);
    pub const UnsupportedManifestVersion: Self = Self(4i32);
    pub const UnsupportedManifestProfile: Self = Self(5i32);
    pub const UnknownFailure: Self = Self(6i32);
}
impl ::core::marker::Copy for AdaptiveMediaSourceCreationStatus {}
impl ::core::clone::Clone for AdaptiveMediaSourceCreationStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AdaptiveMediaSourceDiagnosticAvailableEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Streaming_Adaptive\"`*"]
#[repr(transparent)]
pub struct AdaptiveMediaSourceDiagnosticType(pub i32);
impl AdaptiveMediaSourceDiagnosticType {
    pub const ManifestUnchangedUponReload: Self = Self(0i32);
    pub const ManifestMismatchUponReload: Self = Self(1i32);
    pub const ManifestSignaledEndOfLiveEventUponReload: Self = Self(2i32);
    pub const MediaSegmentSkipped: Self = Self(3i32);
    pub const ResourceNotFound: Self = Self(4i32);
    pub const ResourceTimedOut: Self = Self(5i32);
    pub const ResourceParsingError: Self = Self(6i32);
    pub const BitrateDisabled: Self = Self(7i32);
    pub const FatalMediaSourceError: Self = Self(8i32);
}
impl ::core::marker::Copy for AdaptiveMediaSourceDiagnosticType {}
impl ::core::clone::Clone for AdaptiveMediaSourceDiagnosticType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AdaptiveMediaSourceDiagnostics = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceDownloadBitrateChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Streaming_Adaptive\"`*"]
#[repr(transparent)]
pub struct AdaptiveMediaSourceDownloadBitrateChangedReason(pub i32);
impl AdaptiveMediaSourceDownloadBitrateChangedReason {
    pub const SufficientInboundBitsPerSecond: Self = Self(0i32);
    pub const InsufficientInboundBitsPerSecond: Self = Self(1i32);
    pub const LowBufferLevel: Self = Self(2i32);
    pub const PositionChanged: Self = Self(3i32);
    pub const TrackSelectionChanged: Self = Self(4i32);
    pub const DesiredBitratesChanged: Self = Self(5i32);
    pub const ErrorInPreviousBitrate: Self = Self(6i32);
}
impl ::core::marker::Copy for AdaptiveMediaSourceDownloadBitrateChangedReason {}
impl ::core::clone::Clone for AdaptiveMediaSourceDownloadBitrateChangedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AdaptiveMediaSourceDownloadCompletedEventArgs = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceDownloadFailedEventArgs = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceDownloadRequestedDeferral = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceDownloadRequestedEventArgs = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceDownloadResult = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourceDownloadStatistics = *mut ::core::ffi::c_void;
pub type AdaptiveMediaSourcePlaybackBitrateChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Streaming_Adaptive\"`*"]
#[repr(transparent)]
pub struct AdaptiveMediaSourceResourceType(pub i32);
impl AdaptiveMediaSourceResourceType {
    pub const Manifest: Self = Self(0i32);
    pub const InitializationSegment: Self = Self(1i32);
    pub const MediaSegment: Self = Self(2i32);
    pub const Key: Self = Self(3i32);
    pub const InitializationVector: Self = Self(4i32);
    pub const MediaSegmentIndex: Self = Self(5i32);
}
impl ::core::marker::Copy for AdaptiveMediaSourceResourceType {}
impl ::core::clone::Clone for AdaptiveMediaSourceResourceType {
    fn clone(&self) -> Self {
        *self
    }
}
