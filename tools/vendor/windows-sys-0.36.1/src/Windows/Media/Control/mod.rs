pub type CurrentSessionChangedEventArgs = *mut ::core::ffi::c_void;
pub type GlobalSystemMediaTransportControlsSession = *mut ::core::ffi::c_void;
pub type GlobalSystemMediaTransportControlsSessionManager = *mut ::core::ffi::c_void;
pub type GlobalSystemMediaTransportControlsSessionMediaProperties = *mut ::core::ffi::c_void;
pub type GlobalSystemMediaTransportControlsSessionPlaybackControls = *mut ::core::ffi::c_void;
pub type GlobalSystemMediaTransportControlsSessionPlaybackInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_Control\"`*"]
#[repr(transparent)]
pub struct GlobalSystemMediaTransportControlsSessionPlaybackStatus(pub i32);
impl GlobalSystemMediaTransportControlsSessionPlaybackStatus {
    pub const Closed: Self = Self(0i32);
    pub const Opened: Self = Self(1i32);
    pub const Changing: Self = Self(2i32);
    pub const Stopped: Self = Self(3i32);
    pub const Playing: Self = Self(4i32);
    pub const Paused: Self = Self(5i32);
}
impl ::core::marker::Copy for GlobalSystemMediaTransportControlsSessionPlaybackStatus {}
impl ::core::clone::Clone for GlobalSystemMediaTransportControlsSessionPlaybackStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GlobalSystemMediaTransportControlsSessionTimelineProperties = *mut ::core::ffi::c_void;
pub type MediaPropertiesChangedEventArgs = *mut ::core::ffi::c_void;
pub type PlaybackInfoChangedEventArgs = *mut ::core::ffi::c_void;
pub type SessionsChangedEventArgs = *mut ::core::ffi::c_void;
pub type TimelinePropertiesChangedEventArgs = *mut ::core::ffi::c_void;
