pub type IRemoteSystemFilter = *mut ::core::ffi::c_void;
pub type RemoteSystem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemAccessStatus(pub i32);
impl RemoteSystemAccessStatus {
    pub const Unspecified: Self = Self(0i32);
    pub const Allowed: Self = Self(1i32);
    pub const DeniedByUser: Self = Self(2i32);
    pub const DeniedBySystem: Self = Self(3i32);
}
impl ::core::marker::Copy for RemoteSystemAccessStatus {}
impl ::core::clone::Clone for RemoteSystemAccessStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemAddedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemApp = *mut ::core::ffi::c_void;
pub type RemoteSystemAppRegistration = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemAuthorizationKind(pub i32);
impl RemoteSystemAuthorizationKind {
    pub const SameUser: Self = Self(0i32);
    pub const Anonymous: Self = Self(1i32);
}
impl ::core::marker::Copy for RemoteSystemAuthorizationKind {}
impl ::core::clone::Clone for RemoteSystemAuthorizationKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemAuthorizationKindFilter = *mut ::core::ffi::c_void;
pub type RemoteSystemConnectionInfo = *mut ::core::ffi::c_void;
pub type RemoteSystemConnectionRequest = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemDiscoveryType(pub i32);
impl RemoteSystemDiscoveryType {
    pub const Any: Self = Self(0i32);
    pub const Proximal: Self = Self(1i32);
    pub const Cloud: Self = Self(2i32);
    pub const SpatiallyProximal: Self = Self(3i32);
}
impl ::core::marker::Copy for RemoteSystemDiscoveryType {}
impl ::core::clone::Clone for RemoteSystemDiscoveryType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemDiscoveryTypeFilter = *mut ::core::ffi::c_void;
pub type RemoteSystemEnumerationCompletedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemKindFilter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemPlatform(pub i32);
impl RemoteSystemPlatform {
    pub const Unknown: Self = Self(0i32);
    pub const Windows: Self = Self(1i32);
    pub const Android: Self = Self(2i32);
    pub const Ios: Self = Self(3i32);
    pub const Linux: Self = Self(4i32);
}
impl ::core::marker::Copy for RemoteSystemPlatform {}
impl ::core::clone::Clone for RemoteSystemPlatform {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemRemovedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSession = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionAddedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionController = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionCreationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemSessionCreationStatus(pub i32);
impl RemoteSystemSessionCreationStatus {
    pub const Success: Self = Self(0i32);
    pub const SessionLimitsExceeded: Self = Self(1i32);
    pub const OperationAborted: Self = Self(2i32);
}
impl ::core::marker::Copy for RemoteSystemSessionCreationStatus {}
impl ::core::clone::Clone for RemoteSystemSessionCreationStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemSessionDisconnectedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemSessionDisconnectedReason(pub i32);
impl RemoteSystemSessionDisconnectedReason {
    pub const SessionUnavailable: Self = Self(0i32);
    pub const RemovedByController: Self = Self(1i32);
    pub const SessionClosed: Self = Self(2i32);
}
impl ::core::marker::Copy for RemoteSystemSessionDisconnectedReason {}
impl ::core::clone::Clone for RemoteSystemSessionDisconnectedReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemSessionInfo = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionInvitation = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionInvitationListener = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionInvitationReceivedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionJoinRequest = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionJoinRequestedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionJoinResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemSessionJoinStatus(pub i32);
impl RemoteSystemSessionJoinStatus {
    pub const Success: Self = Self(0i32);
    pub const SessionLimitsExceeded: Self = Self(1i32);
    pub const OperationAborted: Self = Self(2i32);
    pub const SessionUnavailable: Self = Self(3i32);
    pub const RejectedByController: Self = Self(4i32);
}
impl ::core::marker::Copy for RemoteSystemSessionJoinStatus {}
impl ::core::clone::Clone for RemoteSystemSessionJoinStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemSessionMessageChannel = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemSessionMessageChannelReliability(pub i32);
impl RemoteSystemSessionMessageChannelReliability {
    pub const Reliable: Self = Self(0i32);
    pub const Unreliable: Self = Self(1i32);
}
impl ::core::marker::Copy for RemoteSystemSessionMessageChannelReliability {}
impl ::core::clone::Clone for RemoteSystemSessionMessageChannelReliability {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemSessionOptions = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionParticipant = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionParticipantAddedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionParticipantRemovedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionParticipantWatcher = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemSessionParticipantWatcherStatus(pub i32);
impl RemoteSystemSessionParticipantWatcherStatus {
    pub const Created: Self = Self(0i32);
    pub const Started: Self = Self(1i32);
    pub const EnumerationCompleted: Self = Self(2i32);
    pub const Stopping: Self = Self(3i32);
    pub const Stopped: Self = Self(4i32);
    pub const Aborted: Self = Self(5i32);
}
impl ::core::marker::Copy for RemoteSystemSessionParticipantWatcherStatus {}
impl ::core::clone::Clone for RemoteSystemSessionParticipantWatcherStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemSessionRemovedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionUpdatedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionValueSetReceivedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemSessionWatcher = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemSessionWatcherStatus(pub i32);
impl RemoteSystemSessionWatcherStatus {
    pub const Created: Self = Self(0i32);
    pub const Started: Self = Self(1i32);
    pub const EnumerationCompleted: Self = Self(2i32);
    pub const Stopping: Self = Self(3i32);
    pub const Stopped: Self = Self(4i32);
    pub const Aborted: Self = Self(5i32);
}
impl ::core::marker::Copy for RemoteSystemSessionWatcherStatus {}
impl ::core::clone::Clone for RemoteSystemSessionWatcherStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemStatus(pub i32);
impl RemoteSystemStatus {
    pub const Unavailable: Self = Self(0i32);
    pub const DiscoveringAvailability: Self = Self(1i32);
    pub const Available: Self = Self(2i32);
    pub const Unknown: Self = Self(3i32);
}
impl ::core::marker::Copy for RemoteSystemStatus {}
impl ::core::clone::Clone for RemoteSystemStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemStatusType(pub i32);
impl RemoteSystemStatusType {
    pub const Any: Self = Self(0i32);
    pub const Available: Self = Self(1i32);
}
impl ::core::marker::Copy for RemoteSystemStatusType {}
impl ::core::clone::Clone for RemoteSystemStatusType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemStatusTypeFilter = *mut ::core::ffi::c_void;
pub type RemoteSystemUpdatedEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemWatcher = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_RemoteSystems\"`*"]
#[repr(transparent)]
pub struct RemoteSystemWatcherError(pub i32);
impl RemoteSystemWatcherError {
    pub const Unknown: Self = Self(0i32);
    pub const InternetNotAvailable: Self = Self(1i32);
    pub const AuthenticationError: Self = Self(2i32);
}
impl ::core::marker::Copy for RemoteSystemWatcherError {}
impl ::core::clone::Clone for RemoteSystemWatcherError {
    fn clone(&self) -> Self {
        *self
    }
}
pub type RemoteSystemWatcherErrorOccurredEventArgs = *mut ::core::ffi::c_void;
pub type RemoteSystemWebAccountFilter = *mut ::core::ffi::c_void;
