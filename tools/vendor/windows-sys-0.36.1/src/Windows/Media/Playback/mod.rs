#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct AutoLoadedDisplayPropertyKind(pub i32);
impl AutoLoadedDisplayPropertyKind {
    pub const None: Self = Self(0i32);
    pub const MusicOrVideo: Self = Self(1i32);
    pub const Music: Self = Self(2i32);
    pub const Video: Self = Self(3i32);
}
impl ::core::marker::Copy for AutoLoadedDisplayPropertyKind {}
impl ::core::clone::Clone for AutoLoadedDisplayPropertyKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CurrentMediaPlaybackItemChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct FailedMediaStreamKind(pub i32);
impl FailedMediaStreamKind {
    pub const Unknown: Self = Self(0i32);
    pub const Audio: Self = Self(1i32);
    pub const Video: Self = Self(2i32);
}
impl ::core::marker::Copy for FailedMediaStreamKind {}
impl ::core::clone::Clone for FailedMediaStreamKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IMediaEnginePlaybackSource = *mut ::core::ffi::c_void;
pub type IMediaPlaybackSource = *mut ::core::ffi::c_void;
pub type MediaBreak = *mut ::core::ffi::c_void;
pub type MediaBreakEndedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaBreakInsertionMethod(pub i32);
impl MediaBreakInsertionMethod {
    pub const Interrupt: Self = Self(0i32);
    pub const Replace: Self = Self(1i32);
}
impl ::core::marker::Copy for MediaBreakInsertionMethod {}
impl ::core::clone::Clone for MediaBreakInsertionMethod {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaBreakManager = *mut ::core::ffi::c_void;
pub type MediaBreakSchedule = *mut ::core::ffi::c_void;
pub type MediaBreakSeekedOverEventArgs = *mut ::core::ffi::c_void;
pub type MediaBreakSkippedEventArgs = *mut ::core::ffi::c_void;
pub type MediaBreakStartedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaCommandEnablingRule(pub i32);
impl MediaCommandEnablingRule {
    pub const Auto: Self = Self(0i32);
    pub const Always: Self = Self(1i32);
    pub const Never: Self = Self(2i32);
}
impl ::core::marker::Copy for MediaCommandEnablingRule {}
impl ::core::clone::Clone for MediaCommandEnablingRule {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaItemDisplayProperties = *mut ::core::ffi::c_void;
pub type MediaPlaybackAudioTrackList = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManager = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerAutoRepeatModeReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerCommandBehavior = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerFastForwardReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerNextReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerPauseReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerPlayReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerPositionReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerPreviousReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerRateReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerRewindReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackCommandManagerShuffleReceivedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackItem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaPlaybackItemChangedReason(pub i32);
impl MediaPlaybackItemChangedReason {
    pub const InitialItem: Self = Self(0i32);
    pub const EndOfStream: Self = Self(1i32);
    pub const Error: Self = Self(2i32);
    pub const AppRequested: Self = Self(3i32);
}
impl ::core::marker::Copy for MediaPlaybackItemChangedReason {}
impl ::core::clone::Clone for MediaPlaybackItemChangedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaPlaybackItemError = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaPlaybackItemErrorCode(pub i32);
impl MediaPlaybackItemErrorCode {
    pub const None: Self = Self(0i32);
    pub const Aborted: Self = Self(1i32);
    pub const NetworkError: Self = Self(2i32);
    pub const DecodeError: Self = Self(3i32);
    pub const SourceNotSupportedError: Self = Self(4i32);
    pub const EncryptionError: Self = Self(5i32);
}
impl ::core::marker::Copy for MediaPlaybackItemErrorCode {}
impl ::core::clone::Clone for MediaPlaybackItemErrorCode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaPlaybackItemFailedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackItemOpenedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackList = *mut ::core::ffi::c_void;
pub type MediaPlaybackSession = *mut ::core::ffi::c_void;
pub type MediaPlaybackSessionBufferingStartedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlaybackSessionOutputDegradationPolicyState = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaPlaybackSessionVideoConstrictionReason(pub i32);
impl MediaPlaybackSessionVideoConstrictionReason {
    pub const None: Self = Self(0i32);
    pub const VirtualMachine: Self = Self(1i32);
    pub const UnsupportedDisplayAdapter: Self = Self(2i32);
    pub const UnsignedDriver: Self = Self(3i32);
    pub const FrameServerEnabled: Self = Self(4i32);
    pub const OutputProtectionFailed: Self = Self(5i32);
    pub const Unknown: Self = Self(6i32);
}
impl ::core::marker::Copy for MediaPlaybackSessionVideoConstrictionReason {}
impl ::core::clone::Clone for MediaPlaybackSessionVideoConstrictionReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaPlaybackSphericalVideoProjection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaPlaybackState(pub i32);
impl MediaPlaybackState {
    pub const None: Self = Self(0i32);
    pub const Opening: Self = Self(1i32);
    pub const Buffering: Self = Self(2i32);
    pub const Playing: Self = Self(3i32);
    pub const Paused: Self = Self(4i32);
}
impl ::core::marker::Copy for MediaPlaybackState {}
impl ::core::clone::Clone for MediaPlaybackState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaPlaybackTimedMetadataTrackList = *mut ::core::ffi::c_void;
pub type MediaPlaybackVideoTrackList = *mut ::core::ffi::c_void;
pub type MediaPlayer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaPlayerAudioCategory(pub i32);
impl MediaPlayerAudioCategory {
    pub const Other: Self = Self(0i32);
    pub const Communications: Self = Self(3i32);
    pub const Alerts: Self = Self(4i32);
    pub const SoundEffects: Self = Self(5i32);
    pub const GameEffects: Self = Self(6i32);
    pub const GameMedia: Self = Self(7i32);
    pub const GameChat: Self = Self(8i32);
    pub const Speech: Self = Self(9i32);
    pub const Movie: Self = Self(10i32);
    pub const Media: Self = Self(11i32);
}
impl ::core::marker::Copy for MediaPlayerAudioCategory {}
impl ::core::clone::Clone for MediaPlayerAudioCategory {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaPlayerAudioDeviceType(pub i32);
impl MediaPlayerAudioDeviceType {
    pub const Console: Self = Self(0i32);
    pub const Multimedia: Self = Self(1i32);
    pub const Communications: Self = Self(2i32);
}
impl ::core::marker::Copy for MediaPlayerAudioDeviceType {}
impl ::core::clone::Clone for MediaPlayerAudioDeviceType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaPlayerDataReceivedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct MediaPlayerError(pub i32);
impl MediaPlayerError {
    pub const Unknown: Self = Self(0i32);
    pub const Aborted: Self = Self(1i32);
    pub const NetworkError: Self = Self(2i32);
    pub const DecodingError: Self = Self(3i32);
    pub const SourceNotSupported: Self = Self(4i32);
}
impl ::core::marker::Copy for MediaPlayerError {}
impl ::core::clone::Clone for MediaPlayerError {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaPlayerFailedEventArgs = *mut ::core::ffi::c_void;
pub type MediaPlayerRateChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct MediaPlayerState(pub i32);
#[cfg(feature = "deprecated")]
impl MediaPlayerState {
    pub const Closed: Self = Self(0i32);
    pub const Opening: Self = Self(1i32);
    pub const Buffering: Self = Self(2i32);
    pub const Playing: Self = Self(3i32);
    pub const Paused: Self = Self(4i32);
    pub const Stopped: Self = Self(5i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for MediaPlayerState {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for MediaPlayerState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type MediaPlayerSurface = *mut ::core::ffi::c_void;
pub type PlaybackMediaMarker = *mut ::core::ffi::c_void;
pub type PlaybackMediaMarkerReachedEventArgs = *mut ::core::ffi::c_void;
pub type PlaybackMediaMarkerSequence = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct SphericalVideoProjectionMode(pub i32);
impl SphericalVideoProjectionMode {
    pub const Spherical: Self = Self(0i32);
    pub const Flat: Self = Self(1i32);
}
impl ::core::marker::Copy for SphericalVideoProjectionMode {}
impl ::core::clone::Clone for SphericalVideoProjectionMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct StereoscopicVideoRenderMode(pub i32);
impl StereoscopicVideoRenderMode {
    pub const Mono: Self = Self(0i32);
    pub const Stereo: Self = Self(1i32);
}
impl ::core::marker::Copy for StereoscopicVideoRenderMode {}
impl ::core::clone::Clone for StereoscopicVideoRenderMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TimedMetadataPresentationModeChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Playback\"`*"]
#[repr(transparent)]
pub struct TimedMetadataTrackPresentationMode(pub i32);
impl TimedMetadataTrackPresentationMode {
    pub const Disabled: Self = Self(0i32);
    pub const Hidden: Self = Self(1i32);
    pub const ApplicationPresented: Self = Self(2i32);
    pub const PlatformPresented: Self = Self(3i32);
}
impl ::core::marker::Copy for TimedMetadataTrackPresentationMode {}
impl ::core::clone::Clone for TimedMetadataTrackPresentationMode {
    fn clone(&self) -> Self {
        *self
    }
}
