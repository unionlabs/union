pub type AllJoynAboutData = *mut ::core::ffi::c_void;
pub type AllJoynAboutDataView = *mut ::core::ffi::c_void;
pub type AllJoynAcceptSessionJoinerEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynAuthenticationCompleteEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_AllJoyn\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct AllJoynAuthenticationMechanism(pub i32);
#[cfg(feature = "deprecated")]
impl AllJoynAuthenticationMechanism {
    pub const None: Self = Self(0i32);
    pub const SrpAnonymous: Self = Self(1i32);
    pub const SrpLogon: Self = Self(2i32);
    pub const EcdheNull: Self = Self(3i32);
    pub const EcdhePsk: Self = Self(4i32);
    pub const EcdheEcdsa: Self = Self(5i32);
    pub const EcdheSpeke: Self = Self(6i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for AllJoynAuthenticationMechanism {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for AllJoynAuthenticationMechanism {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AllJoynBusAttachment = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_AllJoyn\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct AllJoynBusAttachmentState(pub i32);
#[cfg(feature = "deprecated")]
impl AllJoynBusAttachmentState {
    pub const Disconnected: Self = Self(0i32);
    pub const Connecting: Self = Self(1i32);
    pub const Connected: Self = Self(2i32);
    pub const Disconnecting: Self = Self(3i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for AllJoynBusAttachmentState {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for AllJoynBusAttachmentState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AllJoynBusAttachmentStateChangedEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynBusObject = *mut ::core::ffi::c_void;
pub type AllJoynBusObjectStoppedEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynCredentials = *mut ::core::ffi::c_void;
pub type AllJoynCredentialsRequestedEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynCredentialsVerificationRequestedEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynMessageInfo = *mut ::core::ffi::c_void;
pub type AllJoynProducerStoppedEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynServiceInfo = *mut ::core::ffi::c_void;
pub type AllJoynServiceInfoRemovedEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynSession = *mut ::core::ffi::c_void;
pub type AllJoynSessionJoinedEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynSessionLostEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_AllJoyn\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct AllJoynSessionLostReason(pub i32);
#[cfg(feature = "deprecated")]
impl AllJoynSessionLostReason {
    pub const None: Self = Self(0i32);
    pub const ProducerLeftSession: Self = Self(1i32);
    pub const ProducerClosedAbruptly: Self = Self(2i32);
    pub const RemovedByProducer: Self = Self(3i32);
    pub const LinkTimeout: Self = Self(4i32);
    pub const Other: Self = Self(5i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for AllJoynSessionLostReason {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for AllJoynSessionLostReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AllJoynSessionMemberAddedEventArgs = *mut ::core::ffi::c_void;
pub type AllJoynSessionMemberRemovedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_AllJoyn\"`, `\"deprecated\"`*"]
#[cfg(feature = "deprecated")]
#[repr(transparent)]
pub struct AllJoynTrafficType(pub i32);
#[cfg(feature = "deprecated")]
impl AllJoynTrafficType {
    pub const Unknown: Self = Self(0i32);
    pub const Messages: Self = Self(1i32);
    pub const RawUnreliable: Self = Self(2i32);
    pub const RawReliable: Self = Self(4i32);
}
#[cfg(feature = "deprecated")]
impl ::core::marker::Copy for AllJoynTrafficType {}
#[cfg(feature = "deprecated")]
impl ::core::clone::Clone for AllJoynTrafficType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AllJoynWatcherStoppedEventArgs = *mut ::core::ffi::c_void;
pub type IAllJoynAcceptSessionJoiner = *mut ::core::ffi::c_void;
pub type IAllJoynProducer = *mut ::core::ffi::c_void;
